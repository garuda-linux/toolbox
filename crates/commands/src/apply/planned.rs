//! The plan as the command line shows it before it runs: a list of steps,
//! or JSON Lines under `--json`.

#[cfg(test)]
mod tests;

use std::io::Write as _;

use toolbox_cli::Exit;
use toolbox_cli::args::Confirm;
use toolbox_cli::output::{INDENT, Paint, Row};
use toolbox_plan::Plan;

use super::events;
use crate::out::Out;

/// Shows the plan. `Some` when the command ends here: a dry run, nothing
/// to do, or a step that would ask with nobody to answer, under `--json`
/// or without a `terminal`.
pub(super) fn planned_out(
    paint: Paint,
    plan: &Plan,
    confirm: Confirm,
    terminal: bool,
    out: &mut Out<'_>,
) -> Option<Exit> {
    let typing = plan
        .steps()
        .iter()
        .any(|step| events::needs_typing(&step.action));

    if confirm.json {
        if typing && !confirm.dry_run {
            let line = events::error_with(
                "a step asks questions while it runs",
                Some("leave out --json and answer them in a terminal"),
                None,
            );

            write!(out, "{line}").ok();
            return Some(Exit::NeedsYes);
        }

        write!(out, "{}", events::planned(plan)).ok();
        if confirm.dry_run {
            return Some(Exit::Success);
        }

        if plan.steps().is_empty() {
            write!(out, "{}", events::done(true)).ok();
            return Some(Exit::Success);
        }

        return None;
    }

    if plan.steps().is_empty() {
        writeln!(out, "Nothing to do").ok();
        return Some(Exit::Success);
    }

    write!(out, "{}", planned(paint, plan)).ok();
    if confirm.dry_run {
        let hint = paint.hint("run it without --dry-run to carry it out");
        writeln!(out, "\n{hint}").ok();
        return Some(Exit::Success);
    }

    if typing && !terminal {
        let error = paint.error("a step asks questions while it runs");
        let hint = paint.hint("run it in a terminal and answer them there");
        writeln!(out, "\n{error}\n{hint}").ok();
        return Some(Exit::NeedsYes);
    }

    None
}

pub fn planned(paint: Paint, plan: &Plan) -> String {
    let count = plan.steps().len();
    let summary = if count == 1 {
        "1 step".to_owned()
    } else {
        format!("{count} steps")
    };

    let rows: Vec<Row> = plan
        .steps()
        .iter()
        .flat_map(|step| step.action.command_lines().into_iter().enumerate())
        .map(|(part, line)| match part {
            0 => Row::plain(vec![line]),
            _ => Row::plain(vec![format!("{INDENT}{line}")]),
        })
        .collect();

    paint.list("Planned", Some(&summary), &[], &rows)
}
