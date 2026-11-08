//! `no-comment-semicolon`: no `;` in comment prose. End the sentence and start
//! a new one. Code in backticks and fenced blocks, directives and
//! commented-out code are skipped.

use std::sync::LazyLock;

use regex::Regex;
use tree_sitter::Point;

use crate::finding::Finding;
use crate::pattern::re;
use crate::source::{Kind, Source, Unit, looks_like_code};

pub(super) const RULE: &str = "no-comment-semicolon";

static CODE_REGION: LazyLock<Regex> = LazyLock::new(|| re(r"(?s)```.*?(```|\z)|`[^`\n]*`"));

pub fn check(source: &Source<'_>, findings: &mut Vec<Finding>) {
    for unit in source.units() {
        let text = unit.text();
        if unit.kind() == Kind::Line && looks_like_code(&text) {
            continue;
        }

        let mut masked = vec![false; text.len()];
        for region in CODE_REGION.find_iter(&text) {
            masked
                .iter_mut()
                .skip(region.start())
                .take(region.len())
                .for_each(|byte| *byte = true);
        }

        for (offset, _) in text.match_indices(';') {
            if masked.get(offset).copied().unwrap_or(true) {
                continue;
            }

            findings.push(
                Finding::new(
                    position(&unit, offset),
                    RULE,
                    "Semicolon in comment prose.".to_owned(),
                    "Clauses chained with `;` read like generated text and usually pack two thoughts into one line.",
                )
                .fix("End the sentence and start a new one, or use a comma or \"and\". Drop the second clause if the code already says it."),
            );
        }
    }
}

fn position(unit: &Unit<'_, '_>, offset: usize) -> Point {
    let mut start = 0;

    for line in unit.lines() {
        let end = start + line.text.len();
        if offset <= end {
            return Point::new(line.at.row, line.at.column + offset - start);
        }

        start = end + 1;
    }

    unit.start()
}
