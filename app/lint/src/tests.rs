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
