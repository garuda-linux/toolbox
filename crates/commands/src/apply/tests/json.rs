use std::collections::VecDeque;
use std::sync::Arc;

use toolbox_cli::Exit;
use toolbox_cli::args::Confirm;
use toolbox_plan::protocol::Outcome;
use toolbox_plan::{Plan, UserAction};
use toolbox_services::apply::Launcher;

use super::{Answers, Refusing, helper, plan};
use crate::apply::carry_out;
use crate::out::{Out, Settings};

fn json(yes: bool, dry_run: bool) -> Confirm {
    Confirm {
        yes,
        dry_run,
        json: true,
    }
}

fn carried(plan: Plan, confirm: Confirm, launcher: Arc<dyn Launcher>) -> (Exit, Vec<String>) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut out = Vec::new();
    let mut terminal = Answers {
        terminal: true,
        answers: VecDeque::new(),
    };

    let exit = carry_out(
        plan,
        confirm,
        None,
        runtime.handle(),
        launcher,
        &mut terminal,
        &mut Out::new(&mut out, Settings::default()),
    );

    let text = String::from_utf8(out).unwrap();
    (exit, text.lines().map(str::to_owned).collect())
}

/// Each line is one JSON object, and its `event` field.
fn events(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .map(|line| {
            let value: serde_json::Value = serde_json::from_str(line).unwrap();
            value["event"].as_str().unwrap().to_owned()
        })
        .collect()
}

#[test]
fn a_run_is_json_lines_and_nothing_else() {
    let (exit, lines) = carried(plan(), json(true, false), helper(Outcome::Succeeded));

    assert_eq!(exit, Exit::Success, "{lines:?}");
    assert_eq!(
        events(&lines),
        ["plan", "started", "output", "finished", "done"],
        "{lines:?}"
    );

    // The stand-in helper numbers its step 0, so only the line is checked.
    assert!(
        lines[2].ends_with(r#","line":"keyring refreshed"}"#),
        "{lines:?}"
    );
    assert_eq!(lines[4], r#"{"schema":1,"event":"done","ok":true}"#);
}

#[test]
fn a_failed_step_ends_with_done_not_ok() {
    let failed = Outcome::Failed {
        cause: toolbox_plan::protocol::Failure::ExitCode { code: 1_i32 },
        cleanup_failed: Vec::new(),
    };

    let (exit, lines) = carried(plan(), json(true, false), helper(failed));

    assert_eq!(exit, Exit::Failure, "{lines:?}");
    assert!(lines[3].contains(r#""outcome":"failed""#), "{lines:?}");
    assert_eq!(
        lines.last().unwrap(),
        r#"{"schema":1,"event":"done","ok":false}"#
    );
}

#[test]
fn json_without_yes_or_dry_run_prints_nothing_and_wants_yes() {
    let (exit, lines) = carried(plan(), json(false, false), helper(Outcome::Succeeded));

    assert_eq!(exit, Exit::NeedsYes);
    assert!(lines.is_empty(), "{lines:?}");
}

#[test]
fn a_dry_run_is_the_plan_line_alone() {
    let (exit, lines) = carried(plan(), json(false, true), helper(Outcome::Succeeded));

    assert_eq!(exit, Exit::Success);
    assert_eq!(events(&lines), ["plan"], "{lines:?}");
}

#[test]
fn an_empty_plan_is_planned_and_done() {
    let (exit, lines) = carried(Plan::new([]), json(true, false), helper(Outcome::Succeeded));

    assert_eq!(exit, Exit::Success);
    assert_eq!(
        lines,
        [
            r#"{"schema":1,"event":"plan","steps":[]}"#,
            r#"{"schema":1,"event":"done","ok":true}"#
        ]
    );
}

#[test]
fn a_step_that_asks_while_it_runs_is_refused() {
    let aur = Plan::user(UserAction::UpgradeAur);
    let (exit, lines) = carried(aur.clone(), json(true, false), helper(Outcome::Succeeded));

    assert_eq!(exit, Exit::NeedsYes);
    assert_eq!(events(&lines), ["error"], "{lines:?}");

    let (planned, plan_lines) = carried(aur, json(false, true), helper(Outcome::Succeeded));
    assert_eq!(planned, Exit::Success);
    assert_eq!(events(&plan_lines), ["plan"], "{plan_lines:?}");
}

#[test]
fn the_sudo_note_stays_off_standard_output() {
    let launcher = Arc::new(Refusing {
        words: "Error executing command as another user: No authentication agent found.",
        then: helper(Outcome::Succeeded),
    });

    let (exit, lines) = carried(plan(), json(true, false), launcher);

    assert_eq!(exit, Exit::Success, "{lines:?}");
    assert_eq!(
        events(&lines),
        ["plan", "started", "output", "finished", "done"],
        "{lines:?}"
    );
}
