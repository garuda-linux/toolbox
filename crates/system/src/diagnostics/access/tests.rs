use toolbox_state::diagnostics::{Access, Attempt, Collector, Outcome, Ran};

use super::{access, pacman_log, read_tail};
use crate::diagnostics::testing::{Machine, report};
use crate::diagnostics::tests::DMESG_RESTRICT;

/// `/var/log/pacman.log` of a Lima session.
const LOG: &str = toolbox_state::testing::status::pacman_log::PACMAN_LOG;
/// The last `[PACMAN] Running` line of [`LOG`].
const LAST_RUN: &str =
    "[2026-11-08T17:35:39+0100] [PACMAN] Running 'pacman -S --noconfirm garuda-common-settings'\n";

fn since_last_run() -> &'static str {
    LOG.get(LOG.find(LAST_RUN).unwrap()..).unwrap()
}

/// The Lima setting, and the same file with its 1 changed to 2 and to 0.
#[test]
fn access_follows_the_kernel_setting_and_defaults_to_the_users_rights() {
    let machine = Machine::new();
    let open = Access {
        kernel_log: true,
        journal: true,
    };

    assert_eq!(access(machine.root()), open);

    machine.file("proc/sys/kernel/dmesg_restrict", DMESG_RESTRICT);
    assert!(!access(machine.root()).kernel_log);

    machine.file(
        "proc/sys/kernel/dmesg_restrict",
        &DMESG_RESTRICT.replace('1', "2"),
    );
    assert!(!access(machine.root()).kernel_log);

    machine.file(
        "proc/sys/kernel/dmesg_restrict",
        &DMESG_RESTRICT.replace('1', "0"),
    );
    machine.file("etc/machine-id", "feed\n");
    assert_eq!(access(machine.root()), open);
}

#[test]
fn the_pacman_log_is_read_from_its_last_run() {
    let machine = Machine::new();

    machine.file("var/log/pacman.log", LOG);

    assert!(since_last_run().ends_with("[ALPM] running 'zzz_post.hook'...\n"));
    assert_eq!(
        machine.collect(Collector::PacmanLog),
        report(since_last_run())
    );
    assert_eq!(machine.asked(), Vec::new());
}

#[test]
fn a_system_without_a_pacman_log_has_no_such_report() {
    let machine = Machine::new();

    assert_eq!(machine.collect(Collector::PacmanLog), Outcome::Unavailable);
}

#[test]
fn of_a_huge_pacman_log_only_the_end_is_read() {
    let machine = Machine::new();

    machine.file("var/log/pacman.log", LOG);

    let Attempt::Ran(Ran {
        output,
        errors: _,
        code: _,
        truncated,
    }) = pacman_log(machine.root(), 1000)
    else {
        panic!("the log was not read");
    };

    // The run line lies before the kept part, so the cut is reported.
    assert!(truncated);
    assert!(output.len() <= 1000);
    assert!(output.starts_with("[2026-11-08T17:36:0"), "{output}");
    assert!(LOG.ends_with(&output));

    // A cut that keeps the last run line is not worth a word.
    assert_eq!(
        pacman_log(machine.root(), LOG.len() - 1),
        Attempt::Ran(Ran {
            output: since_last_run().to_owned(),
            errors: String::new(),
            code: Some(0_i32),
            truncated: false,
        })
    );
}

/// The window starts on the `s` of the second line, one byte after the
/// break. That break belongs to the first line, so the second line stays.
#[test]
fn a_cut_on_a_line_start_keeps_that_line() {
    let machine = Machine::new();
    let text = "first line\nsecond line\nthird line\n";
    machine.file("var/log/pacman.log", text);

    let kept = read_tail(&machine.root().join("var/log/pacman.log"), 23).unwrap();

    assert_eq!(kept, ("second line\nthird line\n".to_owned(), true));
}

/// The window starts inside the second line, so that line is dropped and
/// the third one is kept whole.
#[test]
fn a_cut_inside_a_line_drops_that_line() {
    let machine = Machine::new();
    machine.file(
        "var/log/pacman.log",
        "first line\nsecond line\nthird line\n",
    );

    let kept = read_tail(&machine.root().join("var/log/pacman.log"), 20).unwrap();

    assert_eq!(kept, ("third line\n".to_owned(), true));
}
