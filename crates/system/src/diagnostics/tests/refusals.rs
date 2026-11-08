//! A closed kernel log is read through the helper, and pkexec's answers
//! are told apart.

use std::fs;

use toolbox_plan::protocol::ReadKind;
use toolbox_state::diagnostics::{Collector, Outcome, Refusal};

use super::super::testing::{Machine, report};
use super::{DMESG_AS_USER, DMESG_AS_USER_CODE, DMESG_RESTRICT};

/// A machine whose kernel log is closed, as in the Lima VM.
fn restricted() -> Machine {
    let machine = Machine::new();

    machine.file("proc/sys/kernel/dmesg_restrict", DMESG_RESTRICT);
    machine
}

/// The helper prints `stderr` and exits with `code`, as pkexec did.
fn answering(machine: &Machine, stderr: &str, code: &str) {
    machine.file("helper.stderr", stderr);
    machine.helper(&format!(
        "cat '{}' >&2; exit {}",
        machine.root().join("helper.stderr").display(),
        code.trim()
    ));
}

#[test]
fn a_restricted_kernel_log_is_read_through_the_helper() {
    let machine = restricted();

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

/// The last answer carries what `dmesg` said as a user in the Lima VM,
/// as if the helper's own run had failed the same way.
#[test]
fn the_helpers_answers_and_failures_are_told_apart() {
    let machine = restricted();

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

    let answer = serde_json::json!({
        "result": "ran",
        "output": "",
        "errors": DMESG_AS_USER,
        "code": DMESG_AS_USER_CODE.trim().parse::<i32>().unwrap(),
        "truncated": false,
    });

    machine.file("answer.json", &answer.to_string());
    machine.helper(&format!(
        "cat '{}'",
        machine.root().join("answer.json").display()
    ));
    assert_eq!(
        machine.collect(Collector::Dmesg),
        Outcome::Failed("dmesg: read kernel buffer failed: Operation not permitted".to_owned())
    );
}

/// pkexec's answers as captured in the Lima and Garuda desktop VMs. No
/// agent captured gave 126, so that one is made up.
#[test]
fn pkexecs_refusals_are_told_apart() {
    let machine = restricted();

    machine.helper("exit 126");
    assert_eq!(
        machine.collect(Collector::Dmesg),
        Outcome::Refused(Refusal::Dismissed)
    );

    for (stderr, code, refusal) in [
        (
            include_str!("../../pkexec/fixtures/not-authorized.stderr"),
            include_str!("../../pkexec/fixtures/not-authorized.code"),
            Refusal::Denied,
        ),
        (
            include_str!("../../pkexec/fixtures/dismissed.stderr"),
            include_str!("../../pkexec/fixtures/dismissed.code"),
            Refusal::Denied,
        ),
        (
            include_str!("../../pkexec/fixtures/missing-program.stderr"),
            include_str!("../../pkexec/fixtures/missing-program.code"),
            Refusal::HelperMissing,
        ),
        (
            include_str!("../../pkexec/fixtures/no-agent.stderr"),
            include_str!("../../pkexec/fixtures/no-agent.code"),
            Refusal::NoAgent,
        ),
        (
            include_str!("../../pkexec/fixtures/polkit-down.stderr"),
            include_str!("../../pkexec/fixtures/polkit-down.code"),
            Refusal::NoAgent,
        ),
    ] {
        answering(&machine, stderr, code);
        assert_eq!(
            machine.collect(Collector::Dmesg),
            Outcome::Refused(refusal),
            "{stderr}"
        );
    }
}

/// A pkexec without its setuid bit is broken, not a refusal.
#[test]
fn a_broken_pkexec_fails_with_its_words() {
    let machine = restricted();

    answering(
        &machine,
        include_str!("../../pkexec/fixtures/not-setuid.stderr"),
        include_str!("../../pkexec/fixtures/not-setuid.code"),
    );

    assert!(matches!(
        machine.collect(Collector::Dmesg),
        Outcome::Failed(reason) if reason.contains("pkexec must be setuid root")
    ));
}

#[test]
fn a_helper_that_cannot_start_fails_with_a_reason() {
    let machine = restricted();

    assert!(matches!(
        machine.collect(Collector::Dmesg),
        Outcome::Failed(reason) if reason.contains("not installed")
    ));
}

#[test]
fn an_unstartable_helper_fails_with_its_name() {
    let machine = restricted();

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
