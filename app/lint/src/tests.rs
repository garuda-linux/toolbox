use std::path::Path;

use tree_sitter::Point;

use crate::finding::{Finding, Located};

#[test]
fn only_rust_files_are_linted() {
    assert!(crate::source::is_linted(Path::new("app/gui/src/main.rs")));

    for path in ["README.md", "flake.nix"] {
        assert!(!crate::source::is_linted(Path::new(path)), "{path}");
    }
}

#[test]
fn findings_print_one_based_positions_with_why_and_fix() {
    let finding = Finding::new(
        Point { row: 0, column: 4 },
        "semicolon",
        "A semicolon in a comment.".to_owned(),
        "Readers stumble.",
    )
    .fix("End the sentence.");

    let located = Located {
        path: Path::new("crates/x/src/lib.rs"),
        finding: &finding,
    };

    assert_eq!(
        located.to_string(),
        "crates/x/src/lib.rs:1:5: semicolon: A semicolon in a comment.\n    Why: Readers stumble.\n    How to fix: End the sentence."
    );
}

#[test]
fn only_the_repository_root_result_is_skipped() {
    let repo = std::env::temp_dir().join(format!("lint-result-{}", std::process::id()));
    for folder in ["result", "app/src/result"] {
        std::fs::create_dir_all(repo.join(folder)).expect("a folder");
        std::fs::write(repo.join(folder).join("lib.rs"), "fn main() {}\n").expect("a file");
    }

    let mut files = Vec::new();
    crate::collect_files(&repo, &mut files, &repo).expect("the folders are read");

    assert_eq!(files, vec![repo.join("app/src/result/lib.rs")]);
    std::fs::remove_dir_all(&repo).expect("the temp folder goes");
}
