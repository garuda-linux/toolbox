use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::time::{Duration, Instant};

use toolbox_plan::protocol::{MAX_READ_OUTPUT_BYTES, ReadKind};
use toolbox_state::diagnostics::{Access, Collector, Outcome, Refusal};

use super::testing::{Machine, cancel_after, report};
use super::{DiagnosticSource as _, access};
use crate::testing::running_as_root;

/// Room for the notice that says the start of the output was cut.
const NOTICE_SLACK: usize = 100;

#[test]
fn a_missing_program_is_unavailable() {
    let machine = Machine::new();

    assert_eq!(machine.collect(Collector::Inxi), Outcome::Unavailable);
    assert_eq!(machine.collect(Collector::Health), Outcome::Unavailable);
    assert_eq!(
        machine.collect(Collector::SystemdAnalyze),
        Outcome::Unavailable
    );
    assert_eq!(machine.asked(), Vec::new());
}

#[test]
fn a_program_runs_as_the_user_with_english_utf8_text_and_keeps_its_colours() {
    let machine = Machine::new();

    machine.program(
        "garuda-inxi",
        "printf '\\033[1;34mSystem:\\033[0m %s args, %s, %s, %s\\n' \"$#\" \"$LANG\" \"$LC_ALL\" \"$(test -n \"$HOME\" && echo home-kept)\"",
    );

    assert_eq!(
        machine.collect(Collector::Inxi),
        report("\u{1b}[1;34mSystem:\u{1b}[0m 0 args, C.UTF-8, C.UTF-8, home-kept\u{1b}[0m\n")
    );
}

#[test]
fn garuda_healths_exit_code_with_output_is_a_report() {
    let machine = Machine::new();

    machine.program(
        "garuda-health",
        "echo \"1 issue: snapshots are off ($*)\"; echo 'see the wiki' >&2; exit 2",
    );

    assert_eq!(
        machine.collect(Collector::Health),
        report("1 issue: snapshots are off (--forcetty)\n\nsee the wiki\nExit code: 2\n")
    );
}

#[test]
fn a_program_that_fails_without_output_shows_its_error_text() {
    let machine = Machine::new();

    machine.program(
        "garuda-inxi",
        "printf '\\033[31minxi is not installed\\033[0m\\n' >&2; exit 127",
    );

    assert_eq!(
        machine.collect(Collector::Inxi),
        Outcome::Failed("inxi is not installed".to_owned())
    );
}

#[test]
fn systemd_analyze_runs_blame_then_the_critical_chain() {
    let machine = Machine::new();

    machine.program("systemd-analyze", "echo \"ran: $* $SYSTEMD_COLORS\"");

    assert_eq!(
        machine.collect(Collector::SystemdAnalyze),
        report("ran: blame --no-pager 16\n\nran: critical-chain --no-pager 16\n")
    );
}

#[test]
fn the_second_program_does_not_run_after_a_failed_first() {
    let machine = Machine::new();

    machine.program(
        "systemd-analyze",
        "echo \"ran: $1\"; echo 'System has not been booted with systemd' >&2; exit 1",
    );

    assert_eq!(
        machine.collect(Collector::SystemdAnalyze),
        report("ran: blame\n\nSystem has not been booted with systemd\nExit code: 1\n")
    );
}

#[test]
fn an_open_kernel_log_is_read_as_the_user() {
    let machine = Machine::new();

    machine.file("proc/sys/kernel/dmesg_restrict", "0\n");
    machine.program("dmesg", "echo \"[    0.000000] Linux version, $*\"");

    assert_eq!(
        machine.collect(Collector::Dmesg),
        report("[    0.000000] Linux version, --color=always\n")
    );
    assert_eq!(machine.asked(), Vec::new());
}

#[test]
fn a_restricted_kernel_log_is_read_through_the_helper() {
    let machine = Machine::new();

    machine.file("proc/sys/kernel/dmesg_restrict", "1\n");
    machine.program("dmesg", "echo 'the user program must not run'");
    machine.helper(
        "echo \"{\\\"result\\\":\\\"ran\\\",\\\"output\\\":\\\"kernel \\\\u001b[1mlog\\\\u001b[0m via $1 $2\\\\n\\\",\\\"errors\\\":\\\"\\\",\\\"code\\\":0,\\\"truncated\\\":false}\"",
    );

    assert_eq!(
        machine.collect(Collector::Dmesg),
        report("kernel \u{1b}[1mlog\u{1b}[0m via logs dmesg\u{1b}[0m\n")
    );
    assert_eq!(machine.asked(), [ReadKind::Dmesg]);
}

#[test]
fn the_helpers_answers_and_failures_are_told_apart() {
    let machine = Machine::new();

    machine.file("proc/sys/kernel/dmesg_restrict", "1\n");

    machine.helper("echo '{\"result\":\"missing\"}'");
    assert_eq!(machine.collect(Collector::Dmesg), Outcome::Unavailable);

    machine.helper("exit 126");
    assert_eq!(
        machine.collect(Collector::Dmesg),
        Outcome::Refused(Refusal::Dismissed)
    );
    machine.helper("exit 127");
    assert_eq!(
        machine.collect(Collector::Dmesg),
        Outcome::Refused(Refusal::Denied)
    );

    machine.helper("echo 'garuda-toolbox-helper: cannot answer dmesg: boom' >&2; exit 1");
    assert_eq!(
        machine.collect(Collector::Dmesg),
        Outcome::Failed(
            "The helper of Garuda Toolbox failed: garuda-toolbox-helper: cannot answer dmesg: boom"
                .to_owned()
        )
    );

    machine.helper("echo 'not json'");
    assert!(matches!(
        machine.collect(Collector::Dmesg),
        Outcome::Failed(reason) if reason.contains("was not understood")
    ));

    machine.helper(
        "echo '{\"result\":\"ran\",\"output\":\"\",\"errors\":\"dmesg: Operation not permitted\\n\",\"code\":1,\"truncated\":false}'",
    );
    assert_eq!(
        machine.collect(Collector::Dmesg),
        Outcome::Failed("dmesg: Operation not permitted".to_owned())
    );
}

#[test]
fn pkexecs_refusals_are_told_apart() {
    let machine = Machine::new();

    machine.file("proc/sys/kernel/dmesg_restrict", "1\n");

    machine.helper("exit 126");
    assert_eq!(
        machine.collect(Collector::Dmesg),
        Outcome::Refused(Refusal::Dismissed)
    );

    machine.helper(
        "echo 'Error executing command as another user: Not authorized' >&2; echo '' >&2; echo 'This incident has been reported.' >&2; exit 127",
    );
    assert_eq!(
        machine.collect(Collector::Dmesg),
        Outcome::Refused(Refusal::Denied)
    );

    machine.helper(
        "echo 'Cannot run program /usr/lib/garuda-toolbox/garuda-toolbox-helper: No such file or directory' >&2; exit 127",
    );
    assert_eq!(
        machine.collect(Collector::Dmesg),
        Outcome::Refused(Refusal::HelperMissing)
    );

    for no_agent in [
        "echo 'Error executing command as another user: No authentication agent found.' >&2; exit 127",
        "echo 'Error creating textual authentication agent: No controlling terminal' >&2; exit 127",
        "echo 'Error getting authority: Could not connect: No such file or directory' >&2; exit 127",
    ] {
        machine.helper(no_agent);
        assert_eq!(
            machine.collect(Collector::Dmesg),
            Outcome::Refused(Refusal::NoAgent),
            "{no_agent}"
        );
    }
}

#[test]
fn a_helper_that_cannot_start_fails_with_a_reason() {
    let machine = Machine::new();

    machine.file("proc/sys/kernel/dmesg_restrict", "1\n");

    assert!(matches!(
        machine.collect(Collector::Dmesg),
        Outcome::Failed(reason) if reason.contains("not installed")
    ));
}

#[test]
fn a_readable_journal_is_read_as_the_user() {
    let machine = Machine::new();

    machine.file("etc/machine-id", "0123456789abcdef0123456789abcdef\n");
    machine.file(
        "var/log/journal/0123456789abcdef0123456789abcdef/system.journal",
        "",
    );
    machine.program("journalctl", "echo \"journal: $* $SYSTEMD_COLORS\"");

    assert_eq!(
        machine.collect(Collector::Journal),
        report("journal: -e --no-pager 16\n")
    );
    assert_eq!(machine.asked(), Vec::new());
}

#[test]
fn a_journal_the_user_may_not_open_is_read_through_the_helper() {
    // Root opens every file, so the question cannot be asked as root.
    if running_as_root() {
        return;
    }

    let machine = Machine::new();
    let journal = "run/log/journal/0123456789abcdef0123456789abcdef/system.journal";

    machine.file("etc/machine-id", "0123456789abcdef0123456789abcdef\n");
    machine.file(journal, "");
    fs::set_permissions(
        machine.root().join(journal),
        fs::Permissions::from_mode(0o000),
    )
    .unwrap();
    machine.program("journalctl", "echo 'the user program must not run'");
    machine.helper("echo '{\"result\":\"ran\",\"output\":\"system journal\\n\",\"errors\":\"\",\"code\":0,\"truncated\":false}'");

    assert_eq!(
        access(machine.root()),
        Access {
            kernel_log: true,
            journal: false
        }
    );
    assert_eq!(
        machine.collect(Collector::Journal),
        report("system journal\n")
    );
    assert_eq!(machine.asked(), [ReadKind::Journal]);
}

#[test]
fn a_long_output_keeps_its_end_and_says_that_it_was_cut() {
    let machine = Machine::new();

    machine.program(
        "garuda-inxi",
        "i=0; while [ $i -lt 30000 ]; do echo \"$i ................................................................................\"; i=$((i+1)); done",
    );

    let Outcome::Report(text) = machine.collect(Collector::Inxi) else {
        panic!("no report");
    };

    assert!(text.starts_with("[The start of this output was left out"));
    assert!(text.ends_with(&format!("29999 {}\n", ".".repeat(80))));
    assert!(text.len() <= MAX_READ_OUTPUT_BYTES + NOTICE_SLACK);
}

#[test]
fn a_cancel_stops_the_running_program_and_gives_nothing() {
    let machine = Machine::new();

    machine.program("garuda-inxi", "sleep 30");

    let started = Instant::now();
    let cancel = cancel_after(Duration::from_millis(200));

    assert_eq!(machine.source().collect(Collector::Inxi, &cancel), None);
    assert!(started.elapsed() < Duration::from_secs(10));
}

#[test]
fn colour_codes_alone_on_stdout_leave_the_error_text_of_stderr() {
    let machine = Machine::new();

    machine.program(
        "garuda-inxi",
        "printf '\\033[0m\\n'; printf '\\033[31mno display\\033[0m\\n' >&2; exit 1",
    );

    assert_eq!(
        machine.collect(Collector::Inxi),
        Outcome::Failed("no display".to_owned())
    );
}

#[test]
fn a_timed_out_program_names_itself_and_the_limit() {
    assert_eq!(
        super::timed_out(&toolbox_state::diagnostics::Program {
            name: "garuda-inxi",
            arguments: &[],
            environment: &[],
        }),
        toolbox_state::diagnostics::Attempt::Failed(
            "garuda-inxi took longer than 120 seconds".to_owned()
        )
    );
}

#[test]
fn an_unstartable_program_fails_with_its_name() {
    let machine = Machine::new();
    fs::create_dir_all(machine.root().join("bin/garuda-inxi")).unwrap();

    let outcome = machine.collect(Collector::Inxi);

    match &outcome {
        Outcome::Failed(reason) => {
            assert!(
                reason.contains("garuda-inxi could not be started"),
                "{outcome:?}"
            );
        }
        shown @ (Outcome::Report(_) | Outcome::Unavailable | Outcome::Refused(_)) => {
            panic!("an unstartable program is no report: {shown:?}")
        }
    }
}

#[test]
fn an_unstartable_helper_fails_with_its_name() {
    let machine = Machine::new();

    machine.file("proc/sys/kernel/dmesg_restrict", "1\n");
    fs::create_dir_all(machine.root().join("helper")).unwrap();

    let outcome = machine.collect(Collector::Dmesg);

    match &outcome {
        Outcome::Failed(reason) => {
            assert!(reason.contains("could not be started"), "{outcome:?}");
        }
        shown @ (Outcome::Report(_) | Outcome::Unavailable | Outcome::Refused(_)) => {
            panic!("an unstartable helper is no report: {shown:?}")
        }
    }
}

#[test]
fn endings_and_errors_are_described_in_words() {
    use toolbox_exec::{CaptureError, Captured};

    let ended = Captured {
        output: String::new(),
        errors: String::new(),
        code: None,
        truncated: false,
    };

    assert_eq!(super::describe(&Ok(ended)), "ended by a signal");
    assert_eq!(
        super::describe(&Err(CaptureError::TimedOut)),
        "the program took too long"
    );
}
