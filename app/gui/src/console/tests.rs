use toolbox_core::features::console::HelperFailure;
use toolbox_core::features::console::ending::Ending;
use toolbox_plan::StepId;
use toolbox_plan::protocol::{Event, Failure, Mode, Outcome};
use toolbox_system::pkexec::Refusal;
use toolbox_terminal::TerminalSize;

use super::{Console, Status, StepState};
use crate::apply::{Problem, Report};

fn started(console: &mut Console) {
    console.take(Report::PlanStarted {
        steps: vec![
            (StepId::new(1), "pacman -S firefox".to_owned()),
            (StepId::new(2), "systemctl enable sshd".to_owned()),
        ],
    });
}

fn states(console: &Console) -> Vec<StepState> {
    console.steps().iter().map(|step| step.state).collect()
}

#[test]
fn a_new_console_is_idle_and_empty() {
    let console = Console::new();

    assert_eq!(console.status(), Status::Idle);
    assert_eq!(console.steps(), []);
}

#[test]
fn a_started_plan_lists_its_steps_waiting() {
    let mut console = Console::new();
    started(&mut console);

    assert_eq!(console.status(), Status::Running);
    assert_eq!(states(&console), [StepState::Waiting, StepState::Waiting]);
    assert_eq!(console.steps()[1].command, "systemctl enable sshd");
}

#[test]
fn step_events_move_each_step_along() {
    let mut console = Console::new();
    started(&mut console);

    console.take(Report::Step(Event::Started {
        step: StepId::new(1),
    }));
    assert_eq!(states(&console), [StepState::Running, StepState::Waiting]);

    console.take(Report::Step(Event::Finished {
        step: StepId::new(1),
        outcome: Outcome::Succeeded,
    }));
    console.take(Report::Step(Event::Finished {
        step: StepId::new(2),
        outcome: Outcome::Failed {
            cause: Failure::ExitCode { code: 3_i32 },
            cleanup_failed: Vec::new(),
        },
    }));

    assert_eq!(
        states(&console),
        [
            StepState::Succeeded,
            StepState::Failed(Failure::ExitCode { code: 3_i32 })
        ]
    );
}

#[test]
fn output_reaches_the_terminal() {
    let mut console = Console::new();
    started(&mut console);

    console.take(Report::Step(Event::Output {
        step: StepId::new(1),
        data: "resolving dependencies...\r\n".to_owned(),
    }));

    assert!(console.output_text().contains("resolving dependencies"));
}

#[test]
fn the_end_of_the_plan_sets_its_result() {
    let mut console = Console::new();
    started(&mut console);

    console.take(Report::PlanFinished { succeeded: true });

    assert_eq!(console.status(), Status::Succeeded);
}

#[test]
fn a_refused_second_plan_does_not_overwrite_the_running_one() {
    let mut console = Console::new();
    started(&mut console);

    console.take(Report::Busy);
    assert!(console.refused());
    assert!(console.running());
    assert_eq!(console.status(), Status::Running);

    console.take(Report::PlanFinished { succeeded: true });
    assert_eq!(console.status(), Status::Succeeded);
}

#[test]
fn a_plan_runs_from_its_start_to_its_end() {
    let mut console = Console::new();
    assert!(!console.running());

    started(&mut console);
    assert!(console.running());

    console.take(Report::PlanFinished { succeeded: false });
    assert!(!console.running());
}

#[test]
fn the_refusal_notice_ends_with_the_running_plan() {
    let mut console = Console::new();
    started(&mut console);
    console.take(Report::Busy);

    console.take(Report::PlanFinished { succeeded: true });

    assert!(!console.refused());
    assert!(!console.running());
}

#[test]
fn problems_and_the_helpers_reason_are_kept_for_showing() {
    let mut console = Console::new();
    started(&mut console);

    console.take(Report::Step(Event::Rejected {
        reason: "unknown package".to_owned(),
    }));
    console.take(Report::Problem(Problem::Refused(Refusal::Dismissed)));

    assert_eq!(console.rejected(), Some("unknown package"));
    assert_eq!(
        console.problem(),
        Some(&Problem::Refused(Refusal::Dismissed))
    );
}

#[test]
fn a_new_plan_clears_the_last_ones_problem_and_notices() {
    let mut console = Console::new();
    started(&mut console);
    console.take(Report::Problem(Problem::HelperFailed {
        kind: HelperFailure::Ended,
        detail: String::new(),
    }));
    console.take(Report::CancelWaits);
    console.take(Report::PlanFinished { succeeded: false });

    started(&mut console);

    assert_eq!(console.problem(), None);
    assert!(!console.cancel_waits());
    assert_eq!(console.status(), Status::Running);
}

#[test]
fn the_fitting_size_is_clamped_and_reported_once() {
    let mut console = Console::new();

    let first = console.resize(TerminalSize {
        columns: 5,
        rows: 2,
    });

    let again = console.resize(TerminalSize {
        columns: 5,
        rows: 2,
    });

    let first = first.unwrap();
    assert!(first.columns >= 40 && first.rows >= 10, "{first:?}");
    assert_eq!(again, None);
    assert_eq!(console.terminal_mut().size().columns, first.columns);
}

#[test]
fn each_started_plan_has_its_own_number() {
    let mut console = Console::new();
    assert_eq!(console.run(), 0);

    started(&mut console);
    console.take(Report::PlanFinished { succeeded: true });
    assert_eq!(console.run(), 1, "its end keeps the number");

    started(&mut console);
    assert_eq!(console.run(), 2);
}

#[test]
fn the_ending_counts_the_steps_that_had_nothing_to_do() {
    let mut console = Console::new();
    started(&mut console);
    for (step, outcome) in [(1, Outcome::Succeeded), (2, Outcome::Skipped)] {
        console.take(Report::Step(Event::Finished {
            step: StepId::new(step),
            outcome,
        }));
    }

    console.take(Report::PlanFinished { succeeded: true });

    assert_eq!(console.ending(Mode::Run), Ending::Partly { skipped: 1 });
    assert_eq!(console.ending(Mode::DryRun), Ending::Tested);
}

fn output(console: &mut Console, data: &str) {
    console.take(Report::Step(Event::Output {
        step: StepId::new(1),
        data: data.to_owned(),
    }));
}

#[test]
fn the_output_counts_once_a_program_wrote_and_clears_only_after_the_plan() {
    let mut console = Console::new();
    started(&mut console);

    assert!(!console.has_output());
    output(&mut console, "resolving dependencies...\r\n");
    assert!(console.has_output());
    assert_eq!(console.output_text(), "resolving dependencies...");
    assert!(!console.clear_output(), "the running plan's output went");
    assert!(console.has_output());

    console.take(Report::PlanFinished { succeeded: true });

    assert!(console.clear_output());
    assert!(!console.has_output());
    assert_eq!(console.output_text(), "");
    assert_eq!(console.steps(), []);
    assert_eq!(console.status(), Status::Idle);
}

#[test]
fn clearing_the_output_hides_the_last_share_link_too() {
    let mut console = Console::new();
    started(&mut console);
    output(&mut console, "done\r\n");
    console.take(Report::PlanFinished { succeeded: true });
    *console.share_mut() = toolbox_services::share::Upload::Done(toolbox_privatebin::Paste {
        link: "https://bin.example/?a#key".to_owned(),
        delete_link: String::new(),
    });

    assert!(console.clear_output());

    assert_eq!(console.share(), &toolbox_services::share::Upload::Idle);
}

#[test]
fn a_steps_shell_comment_is_left_out_of_its_command() {
    assert_eq!(
        super::without_comment("mkdir -p /run/x # for root alone"),
        "mkdir -p /run/x"
    );
    assert_eq!(
        super::without_comment("echo '# not a comment'"),
        "echo '# not a comment'"
    );
    assert_eq!(super::without_comment("pacman -Syu"), "pacman -Syu");
}
