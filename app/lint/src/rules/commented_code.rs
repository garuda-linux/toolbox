//! `no-commented-out-code`: comments whose text parses as code. A run of `//`
//! lines is judged as one, so a commented-out block is one finding.

use crate::finding::Finding;
use crate::source::{Kind, Source, looks_like_code};

pub(super) const RULE: &str = "no-commented-out-code";

pub fn check(source: &Source<'_>, findings: &mut Vec<Finding>) {
    for unit in source.units() {
        let prose = matches!(unit.kind(), Kind::Line | Kind::Block);
        if prose && looks_like_code(&unit.text()) {
            findings.push(
                Finding::new(
                    unit.start(),
                    RULE,
                    "Commented-out code.".to_owned(),
                    "Dead code in comments rots, confuses readers about what runs, and git already keeps the history.",
                )
                .fix("Delete it. If it documents a deliberate alternative, replace it with a one-line comment that says why."),
            );
        }
    }
}
