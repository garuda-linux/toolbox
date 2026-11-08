//! The plan as the command line shows it before it runs: a list of steps,
//! or JSON Lines under `--json`.

use std::io::Write;

use toolbox_cli::Exit;
use toolbox_cli::args::Confirm;
use toolbox_cli::output::{Paint, Row};
use toolbox_plan::Plan;

use super::events;

/// Shows the plan. `Some` when the command ends here: a dry run, nothing
/// to do, or a step that would ask under `--json`.
pub fn planned_out(
    paint: Paint,
    plan: &Plan,
    confirm: Confirm,
    out: &mut dyn Write,
) -> Option<Exit> {
    if confirm.json {
        let typing = plan
            .steps()
            .iter()
            .any(|step| events::needs_typing(&step.action));

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
        .map(|step| Row::plain(vec![step.action.command_lines().join(" ")]))
        .collect();

    paint.list("Planned", Some(&summary), &[], &rows)
}
