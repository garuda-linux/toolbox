use std::io;
use std::path::PathBuf;

use super::{CatalogError, LoadProblem, Problem};

fn unreadable() -> CatalogError {
    CatalogError::Read {
        path: PathBuf::from("/usr/share/garuda-toolbox/catalog/gaming.toml"),
        error: io::Error::from(io::ErrorKind::NotFound),
    }
}

#[test]
fn every_kind_of_failure_has_its_own_code() {
    let syntax = CatalogError::Syntax {
        source_name: "gaming.toml".to_owned(),
        error: toml::from_str::<toml::Table>("= 3").unwrap_err(),
    };

    let invalid = CatalogError::Invalid {
        source_name: "gaming.toml".to_owned(),
        id: "steam".to_owned(),
        problem: Problem::EmptyTitle,
    };

    assert_eq!(unreadable().code(), "unreadable");
    assert_eq!(syntax.code(), "syntax");
    assert_eq!(invalid.code(), "invalid");
}

#[test]
fn a_load_problem_keeps_the_code_and_the_whole_message() {
    let problem = LoadProblem::from(&unreadable());

    assert_eq!(problem.code, "unreadable");
    assert_eq!(problem.detail, unreadable().to_string());
    assert!(problem.detail.contains("gaming.toml"), "{}", problem.detail);
}

#[test]
fn several_problems_take_the_first_code_and_every_message() {
    let first = LoadProblem {
        code: "syntax",
        detail: "a.toml: line 1".to_owned(),
    };

    let second = LoadProblem {
        code: "unreadable",
        detail: "b.toml: not found".to_owned(),
    };

    assert_eq!(
        LoadProblem::joined(vec![first, second]),
        Some(LoadProblem {
            code: "syntax",
            detail: "a.toml: line 1\nb.toml: not found".to_owned(),
        })
    );
    assert_eq!(LoadProblem::joined(Vec::new()), None);
}
