use std::ffi::OsString;
use std::io;
use std::os::unix::process::ExitStatusExt as _;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::sync::Arc;
use std::time::Duration;

use toolbox_plan::{InvalidName, NameKind, Reason};
use toolbox_state::aur::order::parse_order;
use toolbox_state::bootrepair::parse_lsblk;
use toolbox_system::{InvalidValue, MissingFact, ProbeError, Stop, Subject};

use super::failure;
use crate::i18n::set_language;

fn subjects_of_every_kind() -> Vec<Subject> {
    vec![
        Subject::LocalDatabase,
        Subject::Disks,
        Subject::RunningKernel,
        Subject::RouteList,
        Subject::BuildOrder {
            package: "paru".to_owned(),
        },
        Subject::SampleBuildOrder {
            package: "paru".to_owned(),
        },
    ]
}

fn samples() -> Vec<ProbeError> {
    let listing = parse_lsblk("not a listing").expect_err("a broken listing");
    let order = parse_order("REPO").expect_err("a cut order line");

    vec![
        ProbeError::Command {
            command_line: "pacman -Q".to_owned(),
            status: ExitStatus::from_raw(256),
            stderr: "error: nothing".to_owned(),
        },
        ProbeError::TimedOut {
            command_line: "pacman".to_owned(),
            after: Duration::from_secs(300),
        },
        ProbeError::Stopped {
            command_line: "pacman".to_owned(),
            why: Stop::Cancelled,
        },
        ProbeError::Stopped {
            command_line: "pacman".to_owned(),
            why: Stop::TooLarge,
        },
        ProbeError::Invalid {
            what: Subject::Data("/proc/mounts".to_owned()),
            error: InvalidValue::from(listing),
        },
        ProbeError::Invalid {
            what: Subject::Data("paru".to_owned()),
            error: InvalidValue::from(order),
        },
        ProbeError::Invalid {
            what: Subject::Data("kernel".to_owned()),
            error: InvalidValue::Name(InvalidName::new(
                NameKind::KernelRelease,
                "a b",
                Reason::InvalidCharacter(' '),
            )),
        },
        ProbeError::NotUtf8 {
            what: Subject::Data("the name".to_owned()),
            raw: OsString::from("x"),
        },
        ProbeError::Missing(MissingFact::UserId {
            path: PathBuf::from("/proc/self/status"),
        }),
        ProbeError::Missing(MissingFact::Root {
            path: PathBuf::from("/proc/mounts"),
        }),
        ProbeError::Missing(MissingFact::BootTime),
        ProbeError::Missing(MissingFact::KernelRelease),
        ProbeError::NotAFile {
            path: PathBuf::from("/etc"),
        },
        ProbeError::NulInPath {
            path: PathBuf::from("/data/x"),
        },
        ProbeError::Panicked,
        ProbeError::fake("/etc/x"),
    ]
}

fn names_of_every_kind() -> Vec<NameKind> {
    vec![
        NameKind::Package,
        NameKind::Unit,
        NameKind::User,
        NameKind::Group,
        NameKind::Locale,
        NameKind::KernelModule,
        NameKind::Charset,
        NameKind::DkmsModule,
        NameKind::DkmsVersion,
        NameKind::KernelRelease,
        NameKind::Host,
        NameKind::KernelParameter,
        NameKind::PlymouthTheme,
        NameKind::MenuEntry,
        NameKind::GrubPicture,
        NameKind::GrubTheme,
        NameKind::Disk,
        NameKind::Partition,
        NameKind::Volume,
        NameKind::Mapper,
        NameKind::BootloaderId,
        NameKind::LoaderEntry,
        NameKind::LoaderDefault,
        NameKind::BuildRun,
    ]
}

/// A text the bundle lacks shows its id, so no sample may show one.
#[test]
fn every_probe_failure_has_its_text_in_each_language() {
    for language in ["en-US", "de-DE"] {
        let ctx = egui::Context::default();
        set_language(&ctx, &language.parse().unwrap());

        for error in samples() {
            let text = failure(&ctx, &error).expect("a failure has a text");
            assert!(
                !text.contains("probe-cause-") && !text.contains("name-"),
                "{language}: {text}"
            );
        }
    }
}

#[test]
fn every_name_kind_has_its_text_in_each_language() {
    for language in ["en-US", "de-DE"] {
        let ctx = egui::Context::default();
        set_language(&ctx, &language.parse().unwrap());

        for kind in names_of_every_kind() {
            let error = ProbeError::Invalid {
                what: Subject::Data("a value".to_owned()),
                error: InvalidValue::Name(InvalidName::new(kind, "a b", Reason::Empty)),
            };

            let text = failure(&ctx, &error).expect("a failure has a text");

            assert!(!text.contains("name-"), "{language} {kind:?}: {text}");
        }
    }
}

#[test]
fn a_command_failure_reads_in_german() {
    let ctx = egui::Context::default();
    set_language(&ctx, &"de-DE".parse().unwrap());
    let error = ProbeError::Command {
        command_line: "pacman -Q".to_owned(),
        status: ExitStatus::from_raw(256),
        stderr: "error: nothing".to_owned(),
    };

    assert_eq!(
        failure(&ctx, &error).as_deref(),
        Some("pacman -Q endete mit Exitcode 1: error: nothing")
    );
}

#[test]
fn an_unavailable_source_has_no_failure_text() {
    let ctx = egui::Context::default();

    assert_eq!(failure(&ctx, &ProbeError::Unavailable), None);
}

#[test]
fn every_subject_phrase_has_its_text_in_each_language() {
    for language in ["en-US", "de-DE"] {
        let ctx = egui::Context::default();
        set_language(&ctx, &language.parse().unwrap());

        for subject in subjects_of_every_kind() {
            let error = ProbeError::Io {
                what: subject.clone(),
                error: Arc::new(io::Error::other("x")),
            };

            let text = failure(&ctx, &error).expect("a failure has a text");

            assert!(
                !text.contains("probe-subject-"),
                "{language} {subject}: {text}"
            );
        }
    }
}

#[test]
fn a_subject_phrase_reads_in_german() {
    let ctx = egui::Context::default();
    set_language(&ctx, &"de-DE".parse().unwrap());
    let error = ProbeError::Io {
        what: Subject::BuildOrder {
            package: "paru".to_owned(),
        },
        error: Arc::new(io::Error::other("kaputt")),
    };

    assert_eq!(
        failure(&ctx, &error).as_deref(),
        Some("die Baureihenfolge von paru: kaputt")
    );
}

#[test]
fn a_subject_phrase_reads_in_english() {
    let ctx = egui::Context::default();
    let error = ProbeError::Io {
        what: Subject::RunningKernel,
        error: Arc::new(io::Error::other("kaputt")),
    };

    assert_eq!(
        failure(&ctx, &error).as_deref(),
        Some("the running kernel: kaputt")
    );
}

/// What a command printed is outside text: an escape sequence or a
/// direction mark in it would repaint or reorder the tooltip.
#[test]
fn hidden_characters_of_outside_text_stay_out_of_the_failure_text() {
    let ctx = egui::Context::default();
    let error = ProbeError::Command {
        command_line: "pacman -Q".to_owned(),
        status: ExitStatus::from_raw(256_i32),
        stderr: "error: \u{1b}[2Jnothing\u{202e}".to_owned(),
    };

    let shown = failure(&ctx, &error).unwrap();

    assert!(!shown.contains('\u{1b}'), "{shown:?}");
    assert!(!shown.contains('\u{202e}'), "{shown:?}");
    assert!(shown.contains("nothing"));
}
