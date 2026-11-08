//! `no-variant-label`: a comment that names an escape or match
//! variant in backticks without giving a reason.

use std::sync::LazyLock;

use regex::Regex;

use super::{excerpt, flatten};
use crate::finding::Finding;
use crate::pattern::re;
use crate::source::{Kind, Source, Unit};
use crate::words::WHY_WORDS;

pub(super) const RULE: &str = "no-variant-label";

static VARIANT: LazyLock<Regex> = LazyLock::new(|| re("`[^`]*`"));

pub fn check(source: &Source<'_>, findings: &mut Vec<Finding>) {
    for unit in source.units() {
        if !matches!(unit.kind(), Kind::Line | Kind::Block) {
            continue;
        }

        let text = flatten(&unit.text());
        if !VARIANT.is_match(&text) || WHY_WORDS.is_match(&text) {
            continue;
        }

        if !Source::in_function_body(unit.first().node) {
            continue;
        }

        if !above_match_arm(source, &unit) {
            continue;
        }

        findings.push(
            Finding::new(
                unit.start(),
                RULE,
                format!(
                    "Comment \"{}\" labels a variant the arm names.",
                    excerpt(&text)
                ),
                "Backticked variant names restate the match arm and go stale when the arm changes.",
            )
            .fix("Delete it, or rewrite to the reason the variant needs special handling."),
        );
    }
}

fn above_match_arm(source: &Source<'_>, unit: &Unit<'_, '_>) -> bool {
    let row = unit.last().end.row + 1;
    let line = source.lines.get(row).copied().unwrap_or_default();
    let trimmed = line.trim_start();
    trimmed.starts_with('\'') && trimmed.contains("=>")
}
