use std::io;
use std::path::PathBuf;

use super::{CatalogError, FileProblem, LoadCode, LoadProblem, Problem};

fn unreadable() -> CatalogError {
    CatalogError::Read {
        path: PathBuf::from("/usr/share/garuda-toolbox/catalog/gaming.toml"),
        error: io::Error::from(io::ErrorKind::NotFound),
    }
}

fn syntax_file(error: &str) -> FileProblem {
    FileProblem::Syntax {
        source_name: "a.toml".to_owned(),
        error: error.to_owned(),
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

    assert_eq!(unreadable().code(), LoadCode::Unreadable);
    assert_eq!(syntax.code(), LoadCode::Syntax);
    assert_eq!(invalid.code(), LoadCode::Invalid);
}

#[test]
fn a_load_problem_keeps_the_code_and_the_file_with_its_words() {
    let problem = LoadProblem::from(&unreadable());

    assert_eq!(problem.code, LoadCode::Unreadable);
    assert_eq!(
        problem.problems,
        [FileProblem::Read {
            path: PathBuf::from("/usr/share/garuda-toolbox/catalog/gaming.toml"),
            error: io::Error::from(io::ErrorKind::NotFound).to_string(),
        }]
    );
}

#[test]
fn several_problems_take_the_first_code_and_every_file() {
    let first = LoadProblem {
        code: LoadCode::Syntax,
        problems: vec![syntax_file("line 1")],
    };

    let second = LoadProblem {
        code: LoadCode::Unreadable,
        problems: vec![FileProblem::Read {
            path: PathBuf::from("b.toml"),
            error: "not found".to_owned(),
        }],
    };

    assert_eq!(
        LoadProblem::joined(vec![first, second]),
        Some(LoadProblem {
            code: LoadCode::Syntax,
            problems: vec![
                syntax_file("line 1"),
                FileProblem::Read {
                    path: PathBuf::from("b.toml"),
                    error: "not found".to_owned(),
                },
            ],
        })
    );
    assert_eq!(LoadProblem::joined(Vec::new()), None);
}
