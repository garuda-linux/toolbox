//! `no-shell-command`: `Command::new` on a shell. Shipped code runs argv
//! only, so no text is ever parsed as a command line.

use crate::finding::Finding;
use crate::source::{Place, Source, spawned_literals};

pub(super) const RULE: &str = "no-shell-command";

const SHELLS: [&str; 2] = ["sh", "bash"];

pub fn check(source: &Source<'_>, place: &Place<'_>, findings: &mut Vec<Finding>) {
    if place.is_test() {
        return;
    }

    for literal in spawned_literals(source) {
        let name = literal.value.rsplit('/').next().unwrap_or_default();
        if !SHELLS.contains(&name) {
            continue;
        }

        findings.push(
            Finding::new(
                literal.at,
                RULE,
                format!("`Command::new(\"{}\")` starts a shell.", literal.value),
                "A shell parses its text, so a name with a space, `;` or `$` becomes a second command.",
            )
            .fix("Run the program itself with its arguments as argv, through `toolbox_plan::procedure::Program`."),
        );
    }
}
