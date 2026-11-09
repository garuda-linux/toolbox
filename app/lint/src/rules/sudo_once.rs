//! `sudo-once`: a string literal that is `sudo` or a path to it, outside
//! the launcher. sudo is only the command line's fallback for pkexec, and
//! one place keeps its rules.

use crate::finding::Finding;
use crate::source::{Place, Source, string_literals};

pub(super) const RULE: &str = "sudo-once";

const LAUNCHER: &str = "crates/services/src/apply/launcher.rs";

pub fn check(source: &Source<'_>, place: &Place<'_>, findings: &mut Vec<Finding>) {
    if place.is_test() || place.is(LAUNCHER) {
        return;
    }

    for literal in string_literals(source) {
        // toolbox-lint: allow(sudo-once) The rule's own pattern.
        if literal.value != "sudo" && !literal.value.ends_with("/sudo") {
            continue;
        }

        findings.push(
            Finding::new(
                literal.at,
                RULE,
                format!("\"{}\" outside the launcher.", literal.value),
                "sudo may only stand in for pkexec on a terminal, and only the launcher checks that.",
            )
            .fix(format!("Go through `{LAUNCHER}`, or add `// toolbox-lint: allow(sudo-once)` with the reason when the text is no program, such as a group name.")),
        );
    }
}
