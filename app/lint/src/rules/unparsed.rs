//! `unparsed`: tree-sitter gave no syntax tree, so no rule could run.
//! Only for a missing tree, never for error nodes: broken code still
//! parses, and the rules read what it holds.

use tree_sitter::Point;

use crate::finding::Finding;

pub(super) const RULE: &str = "unparsed";

pub fn missing() -> Finding {
    Finding::new(
        Point { row: 0, column: 0 },
        RULE,
        "The file was not parsed.".to_owned(),
        "Without a syntax tree no rule can check anything.",
    )
    .fix("Check the file for an encoding problem and run the lint again.")
}
