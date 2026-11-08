//! `separate-statements`: a multiline `let` or a closing block needs a blank
//! line before the next statement. Cramped steps blur together and read like
//! generated text.

use crate::finding::Finding;
use crate::source::Source;

pub(super) const RULE: &str = "separate-statements";

pub fn check(source: &Source<'_>, findings: &mut Vec<Finding>) {
    for block in source.nodes_of_kind("block") {
        let mut cursor = block.walk();
        let children: Vec<_> = block.named_children(&mut cursor).collect();
        for (prev, next) in children.iter().zip(children.iter().skip(1)) {
            if is_ignorable(prev.kind()) || is_ignorable(next.kind()) {
                continue;
            }

            if !needs_space(source, *prev) {
                continue;
            }

            if next.start_position().row == prev.end_position().row + 1 {
                findings.push(
                    Finding::new(
                        next.start_position(),
                        RULE,
                        format!(
                            "Multiline {} touches the next statement.",
                            describe(prev.kind())
                        ),
                        "Cramped steps blur together and hide where one step ends.",
                    )
                    .fix("Add a blank line between them."),
                );
            }
        }
    }
}

/// A multiline `let` always, or a statement closing with a block (`if`, `for`,
/// `match` and the like). Builder chains and calls ending with a semicolon stay grouped.
fn needs_space(source: &Source<'_>, prev: tree_sitter::Node<'_>) -> bool {
    if prev.start_position().row == prev.end_position().row {
        return false;
    }

    if prev.kind() == "let_declaration" {
        return true;
    }

    source.node_text(prev).trim_end().ends_with('}')
}

/// Comments already separate the steps around them and attributes belong to
/// the statement below them, so neither side of those pairs is reported.
fn is_ignorable(kind: &str) -> bool {
    matches!(
        kind,
        "line_comment" | "block_comment" | "attribute_item" | "inner_attribute_item"
    )
}

fn describe(kind: &str) -> &'static str {
    if kind == "let_declaration" {
        "`let`"
    } else {
        "block"
    }
}
