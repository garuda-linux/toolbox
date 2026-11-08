//! `blank-before-comment`: a `//` or `/* */` comment on its own line between
//! statements or items needs a blank line above it, so it reads as the heading
//! of the step below. Comments inside lists, arguments, match arms and other
//! expressions label the element next to them and stay packed. At the start of
//! a block and under another comment a blank line would only be noise. Under an
//! attribute it would split the attribute from its item.

use crate::finding::Finding;
use crate::rules::excerpt;
use crate::source::{Kind, Source};

pub(super) const RULE: &str = "blank-before-comment";

pub fn check(source: &Source<'_>, findings: &mut Vec<Finding>) {
    for unit in source.units() {
        let first = unit.first();
        if !matches!(unit.kind(), Kind::Line | Kind::Block) || !first.alone {
            continue;
        }

        let between_steps = first
            .node
            .parent()
            .is_some_and(|parent| holds_steps(parent.kind()));

        let after_attribute = first
            .node
            .prev_named_sibling()
            .is_some_and(|node| node.kind() == "attribute_item");

        if !between_steps || after_attribute {
            continue;
        }

        let start = unit.start();
        let Some(above) = start.row.checked_sub(1) else {
            continue;
        };

        let line_above = source.lines.get(above).copied().unwrap_or_default();
        if may_touch(line_above.trim()) {
            continue;
        }

        findings.push(
            Finding::new(
                start,
                RULE,
                format!(
                    "Comment \"{}\" touches the line above.",
                    excerpt(&unit.text())
                ),
                "A comment glued to the code above reads as part of it, not as the heading of the step below.",
            )
            .fix("Add a blank line above the comment."),
        );
    }
}

fn holds_steps(kind: &str) -> bool {
    matches!(kind, "block" | "source_file" | "declaration_list")
}

fn may_touch(above: &str) -> bool {
    above.is_empty()
        || above.ends_with(['{', '(', '['])
        || above.starts_with("//")
        || above.ends_with("*/")
}
