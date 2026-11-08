use toolbox_plan::protocol::ReadKind;

use super::{
    Access, Attempt, Collector, Outcome, Ran, Refusal, Selection, Source, full_report, outcome,
    since_last_pacman_run,
};

const OPEN: Access = Access {
    kernel_log: true,
    journal: true,
};

const CLOSED: Access = Access {
    kernel_log: false,
    journal: false,
};

/// `/var/log/pacman.log` of the Lima VM.
const PACMAN_LOG: &str = include_str!("../../../system/src/status/pacman_log/fixtures/pacman.log");
/// The last 1500 lines of it on the installed Garuda system.
const INSTALLED_PACMAN_LOG: &str =
    include_str!("../../../system/src/status/pacman_log/fixtures/installed-pacman.log");

fn ran(output: &str, errors: &str, code: i32) -> Attempt {
    Attempt::Ran(Ran {
        output: output.to_owned(),
        errors: errors.to_owned(),
        code: Some(code),
        truncated: false,
    })
}

fn argv(source: Source) -> Vec<Vec<&'static str>> {
    match source {
        Source::Programs(programs) => programs
            .iter()
            .map(|program| {
                std::iter::once(program.name)
                    .chain(program.arguments.iter().copied())
                    .collect()
            })
            .collect(),
        Source::Helper(_) | Source::PacmanLog => Vec::new(),
    }
}

#[test]
fn the_user_programs_run_as_fixed_argv() {
    assert_eq!(argv(Collector::Inxi.source(CLOSED)), [vec!["garuda-inxi"]]);
    assert_eq!(
        argv(Collector::Health.source(CLOSED)),
        [vec!["garuda-health", "--forcetty"]]
    );
    assert_eq!(
        argv(Collector::SystemdAnalyze.source(CLOSED)),
        [
            vec!["systemd-analyze", "blame", "--no-pager"],
            vec!["systemd-analyze", "critical-chain", "--no-pager"],
        ]
    );
    assert_eq!(
        argv(Collector::Journal.source(OPEN)),
        [vec!["journalctl", "-e", "--no-pager"]]
    );
    assert_eq!(
        argv(Collector::Dmesg.source(OPEN)),
        [vec!["dmesg", "--color=always"]]
    );
    assert_eq!(Collector::PacmanLog.source(CLOSED), Source::PacmanLog);
    assert_eq!(Collector::PacmanLog.source(OPEN), Source::PacmanLog);
}

#[test]
fn root_is_asked_only_for_what_the_user_may_not_read() {
    assert_eq!(
        Collector::Dmesg.source(CLOSED),
        Source::Helper(ReadKind::Dmesg)
    );
    assert_eq!(
        Collector::Journal.source(CLOSED),
        Source::Helper(ReadKind::Journal)
    );

    let mixed = Access {
        kernel_log: false,
        journal: true,
    };

    assert_eq!(
        Collector::Dmesg.source(mixed),
        Source::Helper(ReadKind::Dmesg)
    );
    assert!(matches!(
        Collector::Journal.source(mixed),
        Source::Programs(_)
    ));

    for collector in Collector::ALL {
        assert!(
            !matches!(collector.source(OPEN), Source::Helper(_)),
            "{} asks for root on an open system",
            collector.as_str()
        );
    }
}

#[test]
fn the_helper_runs_the_same_command_the_user_would() {
    for (collector, kind) in [
        (Collector::Dmesg, ReadKind::Dmesg),
        (Collector::Journal, ReadKind::Journal),
    ] {
        let (program, arguments) = kind.program().unwrap();
        let as_user = argv(collector.source(OPEN));
        let command = as_user.first().unwrap();

        assert_eq!(program.name(), *command.first().unwrap());
        assert_eq!(arguments, command.get(1..).unwrap());
        assert_eq!(kind.environment(), environment(collector.source(OPEN)));
    }
}

#[test]
fn ids_are_stable_and_all_is_the_full_report() {
    let ids: Vec<&str> = Collector::ALL.iter().map(|one| one.as_str()).collect();

    assert_eq!(
        ids,
        [
            "inxi",
            "systemd-analyze",
            "journalctl",
            "pacman",
            "dmesg",
            "garuda-health"
        ]
    );

    for collector in Collector::ALL {
        assert_eq!(
            Selection::from_id(collector.as_str()),
            Some(Selection::One(collector))
        );
        assert_eq!(Selection::One(collector).collectors(), [collector]);
    }

    assert_eq!(Selection::from_id("all"), Some(Selection::All));
    assert_eq!(Selection::All.as_str(), "all");
    assert_eq!(Selection::All.collectors(), Collector::ALL);

    for id in ["", "ALL", "inxi ", "upload", "rm -rf /", "full-logs"] {
        assert_eq!(Selection::from_id(id), None, "{id}");
    }
}

#[test]
fn output_of_a_successful_program_is_the_report_without_its_errors() {
    assert_eq!(
        outcome(&[ran("line one\nline two\n", "a hint on stderr\n", 0)]),
        Outcome::Report("line one\nline two\n".to_owned())
    );
    assert_eq!(outcome(&[ran("", "", 0)]), Outcome::Report(String::new()));
}

#[test]
fn a_failing_program_with_output_is_a_report_with_a_notice() {
    assert_eq!(
        outcome(&[ran("2 issues found\n\n", "warning: snapshots missing\n", 3)]),
        Outcome::Report("2 issues found\n\nwarning: snapshots missing\nExit code: 3\n".to_owned())
    );
    assert_eq!(
        outcome(&[ran("findings", "", 1)]),
        Outcome::Report("findings\n\nExit code: 1\n".to_owned())
    );
}

#[test]
fn a_failing_program_without_output_fails_with_its_own_words() {
    assert_eq!(
        outcome(&[ran(
            " \n",
            "dmesg: read kernel buffer failed: Operation not permitted\n",
            1
        )]),
        Outcome::Failed("dmesg: read kernel buffer failed: Operation not permitted".to_owned())
    );
    assert_eq!(
        outcome(&[ran("", "", 4)]),
        Outcome::Failed("Exit code: 4".to_owned())
    );
    assert_eq!(
        outcome(&[Attempt::Ran(Ran::default())]),
        Outcome::Failed("Ended by a signal.".to_owned())
    );
}

#[test]
fn missing_refused_and_broken_programs_are_told_apart() {
    assert_eq!(outcome(&[Attempt::Missing]), Outcome::Unavailable);
    assert_eq!(
        outcome(&[Attempt::Refused(Refusal::Dismissed)]),
        Outcome::Refused(Refusal::Dismissed)
    );
    assert_eq!(
        outcome(&[Attempt::Failed("cannot start".to_owned())]),
        Outcome::Failed("cannot start".to_owned())
    );
    assert_eq!(outcome(&[]), Outcome::Failed("No program ran.".to_owned()));
}

#[test]
fn a_chain_joins_its_outputs_and_keeps_the_last_exit_code() {
    assert_eq!(
        outcome(&[ran("blame", "", 0), ran("chain\n", "", 0)]),
        Outcome::Report("blame\n\nchain\n".to_owned())
    );
    assert_eq!(
        outcome(&[
            ran("blame\n", "", 0),
            ran("", "Bootup is not yet finished\n", 1)
        ]),
        Outcome::Report("blame\n\nBootup is not yet finished\nExit code: 1\n".to_owned())
    );
    assert_eq!(
        outcome(&[ran("blame\n", "", 0), Attempt::Missing]),
        Outcome::Report("blame\n\nThe next program is not installed.\nExit code: 1\n".to_owned())
    );
}

#[test]
fn a_cut_output_says_so_in_its_first_line() {
    let cut = Attempt::Ran(Ran {
        output: "newest lines\n".to_owned(),
        errors: String::new(),
        code: Some(0_i32),
        truncated: true,
    });

    let Outcome::Report(text) = outcome(&[cut]) else {
        panic!("no report");
    };

    assert!(text.starts_with("[The start of this output was left out"));
    assert!(text.ends_with("]\nnewest lines\n"));
}

#[test]
fn the_full_report_has_every_part_under_its_heading_in_order() {
    let report = full_report(&[
        (Collector::Inxi, Outcome::Unavailable),
        (
            Collector::SystemdAnalyze,
            Outcome::Report("no newline at the end".to_owned()),
        ),
        (Collector::Journal, Outcome::Refused(Refusal::Denied)),
        (Collector::PacmanLog, Outcome::Report("log\n".to_owned())),
        (Collector::Dmesg, Outcome::Failed("no rights".to_owned())),
        (Collector::Health, Outcome::Report(String::new())),
    ]);

    assert_eq!(
        report,
        "### inxi ###\nNot available on this system.\n### END inxi ###\n\n\
         ### systemd-analyze ###\nno newline at the end\n### END systemd-analyze ###\n\n\
         ### journalctl ###\nNot collected: root rights were not given.\n### END journalctl ###\n\n\
         ### pacman ###\nlog\n### END pacman ###\n\n\
         ### dmesg ###\nFailed: no rights\n### END dmesg ###\n\n\
         ### garuda-health ###\n### END garuda-health ###\n\n"
    );
    assert_eq!(full_report(&[]), "");
}

#[test]
fn the_pacman_log_starts_at_the_last_pacman_run() {
    let (part, cut) = since_last_pacman_run(PACMAN_LOG);

    assert!(cut);
    assert!(
        part.starts_with(
            "[2026-11-08T17:35:39+0100] [PACMAN] Running 'pacman -S --noconfirm garuda-common-settings'\n"
        ),
        "{part}"
    );
    assert_eq!(part.lines().count(), 91);
    assert!(PACMAN_LOG.ends_with(part));

    let (installed_part, installed_cut) = since_last_pacman_run(INSTALLED_PACMAN_LOG);

    assert!(installed_cut);
    assert!(
        installed_part
            .starts_with("[2026-11-07T13:48:27+0100] [PACMAN] Running 'pacman -Rns linux-zen'\n"),
        "{installed_part}"
    );
}

/// Edits of the real log: only what follows its last run line, once
/// without the final line break, and that run line with `PACMAN` and
/// `pacman` swapped.
#[test]
fn a_log_without_a_pacman_run_stays_whole() {
    let (part, _cut) = since_last_pacman_run(PACMAN_LOG);
    let (run, after) = part.split_once('\n').unwrap();
    let unterminated = after.strip_suffix('\n').unwrap();
    let swapped = run.replace("[PACMAN] Running 'pacman", "[pacman] Running 'PACMAN");

    assert_eq!(since_last_pacman_run(after), (after, false));
    assert_eq!(since_last_pacman_run(unterminated), (unterminated, false));
    assert_eq!(since_last_pacman_run(""), ("", false));
    assert_eq!(since_last_pacman_run(&swapped), (swapped.as_str(), false));
}

fn environment(source: Source) -> &'static [(&'static str, &'static str)] {
    match source {
        Source::Programs(programs) => programs.first().unwrap().environment,
        Source::Helper(_) | Source::PacmanLog => &[],
    }
}

#[test]
fn every_collector_that_can_is_asked_for_colours() {
    let sixteen: &[(&str, &str)] = &[("SYSTEMD_COLORS", "16")];

    assert_eq!(environment(Collector::Journal.source(OPEN)), sixteen);

    let Source::Programs(analyze) = Collector::SystemdAnalyze.source(OPEN) else {
        panic!("systemd-analyze runs no programs");
    };

    assert!(analyze.iter().all(|program| program.environment == sixteen));
    assert_eq!(environment(Collector::Dmesg.source(OPEN)), []);
    assert_eq!(environment(Collector::Health.source(OPEN)), []);
    assert_eq!(environment(Collector::Inxi.source(OPEN)), []);
}

#[test]
fn colour_codes_stay_in_the_report() {
    assert_eq!(
        outcome(&[ran("\u{1b}[32mok\u{1b}[0m\n", "", 0)]),
        Outcome::Report("\u{1b}[32mok\u{1b}[0m\u{1b}[0m\n".to_owned())
    );
}

#[test]
fn an_open_colour_ends_before_notes_and_the_next_program() {
    assert_eq!(
        outcome(&[ran("\u{1b}[31mfindings\n\n", "\u{1b}[1mbad\n", 2)]),
        Outcome::Report(
            "\u{1b}[31mfindings\u{1b}[0m\n\n\u{1b}[1mbad\u{1b}[0m\nExit code: 2\n".to_owned()
        )
    );
    assert_eq!(
        outcome(&[
            ran("\u{1b}[31mblame", "", 0),
            ran("\u{1b}[32mchain\n", "", 0)
        ]),
        Outcome::Report("\u{1b}[31mblame\u{1b}[0m\n\n\u{1b}[32mchain\u{1b}[0m\n".to_owned())
    );
    assert_eq!(
        outcome(&[ran("\u{1b}[31mblame\n", "", 0), Attempt::Missing]),
        Outcome::Report(
            "\u{1b}[31mblame\u{1b}[0m\n\nThe next program is not installed.\nExit code: 1\n"
                .to_owned()
        )
    );
}

#[test]
fn headings_of_the_full_report_follow_a_colour_reset() {
    let Outcome::Report(journal) = outcome(&[ran("\u{1b}[1;31mfailed", "", 0)]) else {
        panic!("no report");
    };

    let report = full_report(&[
        (Collector::Journal, Outcome::Report(journal)),
        (Collector::PacmanLog, Outcome::Report("log\n".to_owned())),
    ]);

    assert_eq!(
        report,
        "### journalctl ###\n\u{1b}[1;31mfailed\u{1b}[0m\n### END journalctl ###\n\n\
         ### pacman ###\nlog\n### END pacman ###\n\n"
    );
}
