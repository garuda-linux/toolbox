use clap::Parser as _;
use toolbox_cli::args::{Cli, Command};

use super::{NotQueued, Queueable, queueable};
use crate::parity::{AT_ONCE, COMMANDS};

fn command(words: &[&str]) -> Command {
    let argv = std::iter::once("garuda-toolbox").chain(words.iter().copied());
    Cli::try_parse_from(argv).unwrap().command.unwrap()
}

#[test]
fn every_kind_of_change_the_window_queues_can_be_queued_here() {
    for (domain, words) in COMMANDS {
        let typed = command(words);

        assert!(
            queueable(&typed).is_ok(),
            "`{domain}`: `queue add {}` is refused",
            words.join(" ")
        );
    }
}

#[test]
fn what_the_window_runs_at_once_never_waits_in_the_queue() {
    for (what, words) in AT_ONCE {
        let typed = command(words);

        assert!(
            queueable(&typed).is_err(),
            "`{what}`: `queue add {}` is taken",
            words.join(" ")
        );
    }
}

#[test]
fn only_the_unit_file_states_of_a_service_wait_in_the_queue() {
    for queued in ["enable", "disable", "mask", "unmask"] {
        let typed = command(&["services", queued, "sshd.service"]);

        assert!(queueable(&typed).is_ok(), "`services {queued}` is refused");
    }

    for runs_now in ["start", "stop", "restart", "reload"] {
        let typed = command(&["services", runs_now, "sshd.service"]);

        assert_eq!(
            queueable(&typed).err(),
            Some(NotQueued::AtOnce),
            "`services {runs_now}` is queued"
        );
    }
}

#[test]
fn a_maintenance_repair_runs_at_once() {
    queueable(&command(&["maintenance", "run", "clean-package-cache"])).unwrap();
    assert_eq!(
        queueable(&command(&["maintenance", "run", "remote-full-fix"])).unwrap_err(),
        NotQueued::AtOnce
    );
}

#[test]
fn a_read_or_the_flags_of_a_change_run_now_are_refused() {
    assert_eq!(
        queueable(&command(&["apps", "list"])).unwrap_err(),
        NotQueued::Reads
    );
    assert_eq!(
        queueable(&command(&["apps", "install", "firefox", "--yes"])).unwrap_err(),
        NotQueued::Flag
    );
    assert_eq!(
        queueable(&command(&["queue", "apply"])).unwrap_err(),
        NotQueued::AtOnce
    );
}

#[test]
fn wiping_a_home_or_masking_what_the_session_needs_wants_the_name_typed() {
    let wipe = command(&["users", "remove", "guest", "--delete-home"]);
    let keep = command(&["users", "remove", "guest"]);
    let mask = command(&["services", "mask", "dbus.service"]);

    assert_eq!(queueable(&wipe).unwrap().typed().as_deref(), Some("guest"));
    assert_eq!(queueable(&keep).unwrap().typed(), None);
    assert!(matches!(queueable(&mask).unwrap(), Queueable::Unit(_, _)));
}
