//! `program-path`: `Command::new` on a bare program name. A bare name is
//! looked up in `PATH`, which the caller controls, so shipped code names a
//! fixed path through `toolbox_plan::procedure::Program`.

use crate::finding::Finding;
use crate::source::{Place, Source, spawned_literals};

pub(super) const RULE: &str = "program-path";

/// Each file with the one bare name it may start. All of them run as the
/// user, where `PATH` grants nothing the user lacks.
const EXCEPTIONS: [(&str, &str); 2] = [
    // The desktop's own opener, which differs between desktops and distributions.
    ("app/gui/src/links.rs", "xdg-open"),
    // A development tool that runs on the developer's machine, never shipped.
    ("app/xtask/src/apps/wiki.rs", "curl"),
];

pub fn check(source: &Source<'_>, place: &Place<'_>, findings: &mut Vec<Finding>) {
    if place.is_test() {
        return;
    }

    for literal in spawned_literals(source) {
        let listed = EXCEPTIONS
            .iter()
            .any(|(file, program)| place.is(file) && *program == literal.value);

        if literal.value.starts_with('/') || listed {
            continue;
        }

        findings.push(
            Finding::new(
                literal.at,
                RULE,
                format!("`Command::new(\"{}\")` names no path.", literal.value),
                "A bare name runs whatever `PATH` finds first, and the caller sets `PATH`.",
            )
            .fix("Add the program to `toolbox_plan::procedure::Program` with its fixed path, or list the file in `EXCEPTIONS` of `app/lint/src/rules/program_path.rs` with the reason."),
        );
    }
}
