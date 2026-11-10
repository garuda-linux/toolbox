use std::cell::Cell;
use std::time::{Duration, Instant};

use toolbox_cli::output::Style;
use toolbox_core::features::console::HelperFailure;
use toolbox_plan::StepId;
use toolbox_plan::protocol::{Event, Failure, Mode, Outcome};
use toolbox_services::apply::{Problem, Report};
use toolbox_system::pkexec::Refusal;

use super::{Clock, Show};

fn ticking(tick: Duration) -> Clock {
    let start = Instant::now();
    let reads = Cell::new(0_u32);

    Box::new(move || {
        let read = reads.get();
        reads.set(read + 1);
        start + tick * read
    })
}

fn shown(reports: &[Report], dry_run: bool, style: Style) -> String {
    timed(reports, dry_run, style, Duration::from_millis(4_200))
}

fn timed(reports: &[Report], dry_run: bool, style: Style, tick: Duration) -> String {
    let mut show = Show::new(style, Mode::of_dry_run(dry_run), ticking(tick));

    reports.iter().map(|report| show.report(report)).collect()
}

fn started() -> Report {
    Report::PlanStarted {
        steps: vec![(
            StepId::new(0),
            "systemctl restart -- sshd.service".to_owned(),
        )],
    }
}

fn finished(outcome: Outcome) -> Report {
    Report::Step(Event::Finished {
        step: StepId::new(0),
        outcome,
    })
}

#[test]
fn a_run_is_a_section_with_its_steps_and_their_output() {
    let step = StepId::new(0);
    let text = shown(
        &[
            started(),
            Report::Step(Event::Started { step }),
            Report::Step(Event::Output {
                step,
                data: "line one\nline two\n".to_owned(),
            }),
            finished(Outcome::Succeeded),
            Report::PlanFinished { succeeded: true },
        ],
        false,
        Style::Plain,
    );

    assert_eq!(
        text,
        "\nRunning\n\n  [1/1] systemctl restart -- sshd.service\n    line one\n    line two\n  systemctl restart -- sshd.service done (4.2 s)\n\nDone  (12.6 s)\n"
    );
}

#[test]
fn a_failed_step_is_an_error_and_the_run_did_not_finish() {
    let step = StepId::new(0);
    let text = shown(
        &[
            started(),
            Report::Step(Event::Started { step }),
            finished(Outcome::Failed {
                cause: Failure::ExitCode { code: 1_i32 },
                cleanup_failed: Vec::new(),
            }),
            Report::PlanFinished { succeeded: false },
        ],
        false,
        Style::Plain,
    );

    assert_eq!(
        text,
        "\nRunning\n\n  [1/1] systemctl restart -- sshd.service\n  systemctl restart -- sshd.service failed (4.2 s)\n\nerror: the change did not finish\n"
    );
}

#[test]
fn a_helper_that_cannot_start_is_an_error_under_the_running_section() {
    let step = StepId::new(0);
    let text = shown(
        &[
            started(),
            Report::Step(Event::Started { step }),
            Report::Problem(Problem::HelperFailed {
                kind: HelperFailure::Start,
                detail: "/usr/bin/systemctl: no such file".to_owned(),
            }),
            Report::PlanFinished { succeeded: false },
        ],
        false,
        Style::Plain,
    );

    assert_eq!(
        text,
        "\nRunning\n\n  [1/1] systemctl restart -- sshd.service\nerror: the helper did not start\nhint: add --verbose to see the program's own words\n\nerror: the change did not finish\n"
    );
}

#[test]
fn a_test_run_tells_that_nothing_changed() {
    let text = shown(
        &[
            started(),
            finished(Outcome::Skipped),
            Report::PlanFinished { succeeded: true },
        ],
        true,
        Style::Plain,
    );

    assert_eq!(text, "\nTest mode, nothing was changed  (4.2 s)\n");
}

#[test]
fn a_coloured_run_paints_its_sections_and_errors() {
    let step = StepId::new(0);
    let text = shown(
        &[
            started(),
            Report::Step(Event::Started { step }),
            finished(Outcome::Failed {
                cause: Failure::ExitCode { code: 1_i32 },
                cleanup_failed: Vec::new(),
            }),
            Report::PlanFinished { succeeded: false },
        ],
        false,
        Style::Color,
    );

    assert!(
        text.starts_with("\n\u{1b}[1mRunning\u{1b}[0m\n\n    \u{1b}[2m[1/1]\u{1b}[0m systemctl"),
        "{text:?}"
    );
    assert!(
        text.contains("  \u{1b}[31m✗\u{1b}[0m systemctl restart -- sshd.service failed \u{1b}[2m(4.2 s)\u{1b}[0m\n"),
        "{text:?}"
    );
}

#[test]
fn a_coloured_step_that_succeeds_is_marked_green() {
    let step = StepId::new(0);
    let text = shown(
        &[
            started(),
            Report::Step(Event::Started { step }),
            finished(Outcome::Succeeded),
        ],
        false,
        Style::Color,
    );

    assert!(
        text.contains("  \u{1b}[32m●\u{1b}[0m systemctl restart -- sshd.service done \u{1b}[2m(4.2 s)\u{1b}[0m\n"),
        "{text:?}"
    );
}

#[test]
fn a_refusal_says_what_helps_on_a_hint_line() {
    let text = shown(
        &[Report::Problem(Problem::Refused(Refusal::Dismissed))],
        false,
        Style::Plain,
    );

    assert_eq!(
        text,
        "\nerror: not carried out: the password dialog was closed\nhint: run it again and enter your password\n"
    );
}

#[test]
fn each_step_counts_up_and_ends_with_how_long_it_took() {
    let (first, second) = (StepId::new(0), StepId::new(1));
    let text = timed(
        &[
            Report::PlanStarted {
                steps: vec![
                    (first, "pacman-key --refresh-keys".to_owned()),
                    (second, "pacman -Syu".to_owned()),
                ],
            },
            Report::Step(Event::Started { step: first }),
            Report::Step(Event::Finished {
                step: first,
                outcome: Outcome::Succeeded,
            }),
            Report::Step(Event::Started { step: second }),
            Report::Step(Event::Finished {
                step: second,
                outcome: Outcome::Cancelled,
            }),
        ],
        false,
        Style::Plain,
        Duration::from_secs(63),
    );

    assert_eq!(
        text,
        "\nRunning\n\n  [1/2] pacman-key --refresh-keys\n  pacman-key --refresh-keys done (1 min 3 s)\n  [2/2] pacman -Syu\n  pacman -Syu was cancelled (1 min 3 s)\n"
    );
}

#[test]
fn a_step_that_never_started_ends_without_a_time() {
    let text = shown(
        &[started(), finished(Outcome::Cancelled)],
        false,
        Style::Plain,
    );

    assert_eq!(
        text,
        "\nRunning\n\n  systemctl restart -- sshd.service was cancelled\n"
    );
}

/// Ctrl+C mid-line: the note starts a line of its own, the step shows as
/// cancelled and the run closes with how long it ran.
#[test]
fn a_cancelled_run_says_so_and_how_long_it_ran() {
    let step = StepId::new(0);
    let mut show = Show::new(
        Style::Plain,
        Mode::Run,
        ticking(Duration::from_millis(4_200)),
    );

    let mut text = String::new();

    text.push_str(&show.report(&started()));
    text.push_str(&show.report(&Report::Step(Event::Started { step })));
    text.push_str(&show.report(&Report::Step(Event::Output {
        step,
        data: "half a line".to_owned(),
    })));
    text.push_str(&show.note("stopping now"));
    text.push_str(&show.report(&finished(Outcome::Cancelled)));
    text.push_str(&show.report(&Report::PlanFinished { succeeded: false }));

    assert_eq!(
        text,
        "\nRunning\n\n  [1/1] systemctl restart -- sshd.service\n    half a line\nwarning: stopping now\n  systemctl restart -- sshd.service was cancelled (4.2 s)\n\nCancelled  (12.6 s)\n"
    );
}
