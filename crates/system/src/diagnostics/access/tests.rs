use toolbox_state::diagnostics::{Access, Attempt, Collector, Outcome, Ran};

use super::{access, pacman_log};
use crate::diagnostics::testing::{Machine, report};

#[test]
fn access_follows_the_kernel_setting_and_defaults_to_the_users_rights() {
    let machine = Machine::new();
    let open = Access {
        kernel_log: true,
        journal: true,
    };

    assert_eq!(access(machine.root()), open);

    machine.file("proc/sys/kernel/dmesg_restrict", "1\n");
    assert!(!access(machine.root()).kernel_log);

    machine.file("proc/sys/kernel/dmesg_restrict", "2\n");
    assert!(!access(machine.root()).kernel_log);

    machine.file("proc/sys/kernel/dmesg_restrict", "0\n");
    machine.file("etc/machine-id", "feed\n");
    assert_eq!(access(machine.root()), open);
}

#[test]
fn the_pacman_log_is_read_from_its_last_run() {
    let machine = Machine::new();

    machine.file(
        "var/log/pacman.log",
        "[2026-10-01T10:00:00+0200] [PACMAN] Running 'pacman -Syu'\n\
         [2026-10-01T10:00:05+0200] [ALPM] upgraded linux (6.16-1 -> 6.17-1)\n\
         [2026-10-09T08:00:00+0200] [PACMAN] Running 'pacman -S htop'\n\
         [2026-10-09T08:00:03+0200] [ALPM] installed htop (3.4.1-1)\n",
    );

    assert_eq!(
        machine.collect(Collector::PacmanLog),
        report(
            "[2026-10-09T08:00:00+0200] [PACMAN] Running 'pacman -S htop'\n\
             [2026-10-09T08:00:03+0200] [ALPM] installed htop (3.4.1-1)\n"
        )
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
    let filler = "[2026-10-01T10:00:05+0200] [ALPM] upgraded something (1-1 -> 1-2)\n";
    let mut log = String::from("[2026-09-01T10:00:00+0200] [PACMAN] Running 'pacman -Syu'\n");

    log.push_str(&filler.repeat(1000));
    machine.file("var/log/pacman.log", &log);

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
    assert!(output.starts_with("[2026-10-01"), "{output}");
    assert!(output.ends_with(filler));

    log.push_str("[2026-10-09T08:00:00+0200] [PACMAN] Running 'pacman -S htop'\nlast line\n");
    machine.file("var/log/pacman.log", &log);

    assert_eq!(
        pacman_log(machine.root(), 1000),
        Attempt::Ran(Ran {
            output: "[2026-10-09T08:00:00+0200] [PACMAN] Running 'pacman -S htop'\nlast line\n"
                .to_owned(),
            errors: String::new(),
            code: Some(0_i32),
            truncated: false,
        })
    );
}
