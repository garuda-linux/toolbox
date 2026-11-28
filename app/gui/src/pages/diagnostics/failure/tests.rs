use toolbox_core::features::diagnostics::{AnswerFault, Failure, HelperFault, Refusal};

use super::text;
use crate::i18n::set_language;

fn failures_of_every_kind() -> Vec<Failure> {
    vec![
        Failure::NoProgramRan,
        Failure::ExitCode(4),
        Failure::Signal,
        Failure::Stderr("the program's own words".to_owned()),
        Failure::Start {
            program: "garuda-inxi".to_owned(),
            error: "permission denied".to_owned(),
        },
        Failure::TimedOut {
            program: "garuda-inxi".to_owned(),
            seconds: 120,
        },
        Failure::TooLarge {
            program: "journalctl".to_owned(),
        },
        Failure::Read {
            path: "/proc/kmsg".to_owned(),
            error: "denied".to_owned(),
        },
        Failure::Helper(HelperFault::Missing),
        Failure::Helper(HelperFault::Start {
            error: "denied".to_owned(),
        }),
        Failure::Helper(HelperFault::TimedOut),
        Failure::Helper(HelperFault::TooLong),
        Failure::Helper(HelperFault::Pkexec {
            words: "setuid".to_owned(),
        }),
        Failure::Helper(HelperFault::Ended {
            code: Some(1),
            words: "boom".to_owned(),
        }),
        Failure::Helper(HelperFault::Ended {
            code: None,
            words: "boom".to_owned(),
        }),
        Failure::Helper(HelperFault::Unreadable),
        Failure::Helper(HelperFault::Stopped),
        Failure::Answer(AnswerFault::TooLong {
            length: 9,
            limit: 4,
        }),
        Failure::Answer(AnswerFault::Json("expected value".to_owned())),
        Failure::NotInstalled,
        Failure::Refused(Refusal::Dismissed),
        Failure::Refused(Refusal::Denied),
        Failure::Refused(Refusal::NoAgent),
        Failure::Refused(Refusal::HelperMissing),
    ]
}

/// A failure without its text would show the bare id, in each language.
#[test]
fn every_failure_has_its_text_in_each_language() {
    let ctx = egui::Context::default();

    for language in ["en-US", "de-DE"] {
        set_language(&ctx, &language.parse().unwrap());

        for failure in failures_of_every_kind() {
            let shown = text(&ctx, &failure);

            assert!(!shown.contains("diagnostics-"), "{language}: {shown}");
        }
    }
}

#[test]
fn a_helper_failure_reads_in_german() {
    let ctx = egui::Context::default();
    set_language(&ctx, &"de-DE".parse().unwrap());
    let failure = Failure::Helper(HelperFault::Ended {
        code: Some(1_i32),
        words: "boom".to_owned(),
    });

    assert_eq!(
        text(&ctx, &failure),
        "das Hilfsprogramm endete mit Exitcode 1: boom"
    );
}

#[test]
fn a_failure_without_a_program_reads_in_english() {
    let ctx = egui::Context::default();

    assert_eq!(text(&ctx, &Failure::NoProgramRan), "No program ran.");
}
