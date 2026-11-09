mod protected;
mod unknown_fields;

use super::{
    Answer, DecodeError, Event, Failure, MAX_INPUT_BYTES, MAX_LINE_BYTES, MAX_STEPS, Mode, Outcome,
    PROTOCOL_VERSION, ProgramAnswer, ReadKind, Request, RequestError, TerminalSize, decode, encode,
};
use crate::procedure::Program;
use crate::{RootAction, Step, StepId};

fn step(id: u32, action: RootAction) -> Step<RootAction> {
    Step {
        id: StepId::new(id),
        action,
    }
}

fn request(steps: Vec<Step<RootAction>>) -> Request {
    Request::new(Mode::Run, steps)
}

#[test]
fn encoded_messages_are_single_lines() {
    let line = encode(&Event::Started {
        step: StepId::new(3),
    })
    .unwrap();

    assert_eq!(line, "{\"type\":\"started\",\"step\":3}\n");
}

#[test]
fn a_request_round_trips_and_validates() {
    let original = request(vec![step(1, RootAction::RefreshKeyring)]);
    let line = encode(&original).unwrap();
    let parsed = Request::parse(line.trim_end()).unwrap();
    assert_eq!(parsed, original);
    assert_eq!(parsed.mode, Mode::Run);
}

#[test]
fn a_request_from_another_version_is_rejected() {
    let mut other = request(vec![step(1, RootAction::RefreshKeyring)]);
    other.version = PROTOCOL_VERSION + 1;
    let line = encode(&other).unwrap();
    assert_eq!(
        Request::parse(&line).unwrap_err(),
        RequestError::VersionMismatch {
            expected: PROTOCOL_VERSION,
            got: PROTOCOL_VERSION + 1
        }
    );
}

/// A newer app sends actions this helper cannot decode. The version is
/// the reason to name, not the unknown action.
#[test]
fn a_newer_request_with_an_unknown_action_is_a_version_mismatch() {
    let newer = PROTOCOL_VERSION + 1;
    let line = format!(
        r#"{{"version":{newer},"mode":"run","steps":[{{"id":1,"action":{{"kind":"not-invented-yet"}}}}]}}"#
    );

    assert_eq!(
        Request::parse(&line).unwrap_err(),
        RequestError::VersionMismatch {
            expected: PROTOCOL_VERSION,
            got: newer
        }
    );
}

#[test]
fn a_request_without_a_version_is_malformed() {
    for line in [
        r#"{"mode":"run","steps":[]}"#,
        "[]",
        "",
        r#"{"version":"1"}"#,
    ] {
        assert!(
            matches!(Request::parse(line), Err(RequestError::Malformed(_))),
            "{line}"
        );
    }
}

#[test]
fn a_request_without_steps_is_rejected() {
    assert_eq!(
        request(Vec::new()).validate().unwrap_err(),
        RequestError::NoSteps
    );
}

#[test]
fn duplicate_step_ids_are_rejected() {
    let doubled = request(vec![
        step(1, RootAction::RefreshKeyring),
        step(1, RootAction::CleanPackageCache),
    ]);

    assert_eq!(
        doubled.validate().unwrap_err(),
        RequestError::DuplicateStep(StepId::new(1))
    );
}

#[test]
fn too_many_steps_are_rejected() {
    let ids = 1..=u32::try_from(MAX_STEPS + 1).unwrap();
    let many = request(ids.map(|id| step(id, RootAction::RefreshKeyring)).collect());
    assert_eq!(
        many.validate().unwrap_err(),
        RequestError::TooManySteps { max: MAX_STEPS }
    );
}

#[test]
fn an_invalid_action_inside_a_request_fails_to_parse() {
    let line = format!(
        r#"{{"version":{PROTOCOL_VERSION},"mode":"run","steps":[{{"id":1,"action":{{"kind":"enable-units","units":["../evil"]}}}}]}}"#
    );

    assert!(matches!(
        Request::parse(&line),
        Err(RequestError::Malformed(_))
    ));
}

#[test]
fn overlong_lines_are_refused_before_parsing() {
    let line = "x".repeat(MAX_LINE_BYTES + 1);
    assert!(matches!(
        decode::<Answer>(&line),
        Err(DecodeError::TooLong {
            length: _,
            limit: _
        })
    ));
}

#[test]
fn outcomes_and_answers_round_trip() {
    let finished = Event::Finished {
        step: StepId::new(2),
        outcome: Outcome::Failed {
            cause: Failure::ExitCode { code: 1_i32 },
        },
    };

    assert_eq!(
        decode::<Event>(&encode(&finished).unwrap()).unwrap(),
        finished
    );
    let resize = Answer::Resize {
        size: TerminalSize {
            columns: 120,
            rows: 40,
        },
    };

    assert_eq!(decode::<Answer>(&encode(&resize).unwrap()).unwrap(), resize);
    assert_eq!(encode(&Answer::Finish).unwrap(), "{\"type\":\"finish\"}\n");
    assert_eq!(
        decode::<Answer>("{\"type\":\"finish\"}").unwrap(),
        Answer::Finish
    );
}

#[test]
fn a_failure_names_exactly_one_cause_on_the_wire() {
    let outcome = |cause| encode(&Outcome::Failed { cause }).unwrap();

    assert_eq!(
        outcome(Failure::ExitCode { code: 3_i32 }),
        "{\"result\":\"failed\",\"cause\":{\"kind\":\"exit-code\",\"code\":3}}\n"
    );
    assert_eq!(
        outcome(Failure::Signal { signal: 9_i32 }),
        "{\"result\":\"failed\",\"cause\":{\"kind\":\"signal\",\"signal\":9}}\n"
    );
    assert_eq!(
        outcome(Failure::Unknown),
        "{\"result\":\"failed\",\"cause\":{\"kind\":\"unknown\"}}\n"
    );

    for cause in [
        Failure::ExitCode { code: 3_i32 },
        Failure::Signal { signal: 9_i32 },
        Failure::Unknown,
    ] {
        assert_eq!(
            decode::<Outcome>(&outcome(cause)).unwrap(),
            Outcome::Failed { cause }
        );
    }

    assert!(
        decode::<Outcome>(r#"{"result":"failed","code":1,"signal":9}"#).is_err(),
        "a cause with both a code and a signal"
    );
}

#[test]
fn typed_input_is_capped_when_decoded() {
    let input = |length: usize| Answer::Input {
        data: "x".repeat(length),
    };

    assert_eq!(
        decode::<Answer>(&encode(&input(MAX_INPUT_BYTES)).unwrap()).unwrap(),
        input(MAX_INPUT_BYTES)
    );

    let error = decode::<Answer>(&encode(&input(MAX_INPUT_BYTES + 1)).unwrap()).unwrap_err();

    assert!(matches!(error, DecodeError::Json(_)), "{error:?}");
    assert!(!error.to_string().contains("xxxx"), "{error}");
}

#[test]
fn read_kinds_are_a_closed_list_of_plain_words() {
    assert_eq!(
        ReadKind::from_args("read", "shadow-locks"),
        Some(ReadKind::ShadowLocks)
    );

    for kind in ["", "shadow", "/etc/shadow", "shadow-locks ", "SHADOW-LOCKS"] {
        assert_eq!(ReadKind::from_args("read", kind), None, "{kind}");
    }

    for kind in ReadKind::ALL {
        assert_eq!(
            ReadKind::from_args(kind.command(), kind.as_str()),
            Some(kind)
        );
    }
}

/// The menu needs no password in an active session, like the lock flags:
/// it is what the boot screen shows to anyone at the machine.
#[test]
fn the_grub_menu_is_a_read_and_never_a_log() {
    assert_eq!(
        ReadKind::from_args("read", "grub-menu"),
        Some(ReadKind::GrubMenu)
    );
    assert_eq!(ReadKind::from_args("logs", "grub-menu"), None);
    assert_eq!(ReadKind::GrubMenu.program(), None);
}

/// The boot screen shows the same to anyone at the machine.
#[test]
fn the_sdboot_menu_is_a_read_of_files_and_never_a_log() {
    assert_eq!(
        ReadKind::from_args("read", "sdboot-menu"),
        Some(ReadKind::SdBootMenu)
    );
    assert_eq!(ReadKind::from_args("logs", "sdboot-menu"), None);
    assert_eq!(ReadKind::SdBootMenu.program(), None);
}

#[test]
fn program_reads_have_a_fixed_command_and_the_file_read_has_none() {
    assert_eq!(ReadKind::ShadowLocks.program(), None);
    assert_eq!(
        ReadKind::Dmesg.program(),
        Some((Program::Dmesg, ["--color=always"].as_slice()))
    );
    assert_eq!(
        ReadKind::Journal.program(),
        Some((Program::Journalctl, ["-e", "--no-pager"].as_slice()))
    );
    assert_eq!(ReadKind::from_args("logs", "dmesg"), Some(ReadKind::Dmesg));
    assert_eq!(
        ReadKind::from_args("logs", "journal"),
        Some(ReadKind::Journal)
    );

    for kind in ["dmesg -w", "journalctl", "journal -f", "--journal"] {
        assert_eq!(ReadKind::from_args("logs", kind), None, "{kind}");
    }
}

/// polkit lets an active session run `read` without a password. Whatever
/// runs a program as root must not be reachable that way.
#[test]
fn no_kind_that_runs_a_program_belongs_to_the_password_less_command() {
    for kind in ReadKind::ALL {
        assert_eq!(
            kind.command() == "read",
            kind.program().is_none(),
            "{}",
            kind.as_str()
        );
    }

    for (command, kind) in [
        ("read", "dmesg"),
        ("read", "journal"),
        ("logs", "shadow-locks"),
        ("apply", "dmesg"),
        ("", "journal"),
    ] {
        assert_eq!(ReadKind::from_args(command, kind), None, "{command} {kind}");
    }
}

#[test]
fn a_program_answer_is_one_tagged_line() {
    let ran = ProgramAnswer::Ran {
        output: "line\n".to_owned(),
        errors: String::new(),
        code: Some(1_i32),
        truncated: true,
    };

    assert_eq!(
        encode(&ran).unwrap(),
        "{\"result\":\"ran\",\"output\":\"line\\n\",\"errors\":\"\",\"code\":1,\"truncated\":true}\n"
    );
    assert_eq!(
        encode(&ProgramAnswer::Missing).unwrap(),
        "{\"result\":\"missing\"}\n"
    );
    assert_eq!(ProgramAnswer::parse(&encode(&ran).unwrap()).unwrap(), ran);
}

/// A paste can be longer than one input may be. The helper would drop it
/// whole, so the sender cuts it.
#[test]
fn a_long_input_is_sent_in_parts_the_helper_takes() {
    let text = "aé€🦀".repeat(MAX_INPUT_BYTES / 4);
    let parts = Answer::Input { data: text.clone() }.parts();

    assert!(parts.len() > 2, "{}", parts.len());

    let mut joined = String::new();

    for part in &parts {
        let Answer::Input { data } = decode::<Answer>(&encode(part).unwrap()).unwrap() else {
            panic!("a part of an input is an input");
        };

        assert_ne!(data, "");
        joined.push_str(&data);
    }

    assert_eq!(joined, text);
}

#[test]
fn other_answers_and_short_inputs_stay_whole() {
    let short = Answer::Input {
        data: "y\r".to_owned(),
    };

    assert_eq!(short.clone().parts(), [short]);
    assert_eq!(Answer::Cancel.parts(), [Answer::Cancel]);
    assert_eq!(
        Answer::Input {
            data: String::new()
        }
        .parts(),
        [Answer::Input {
            data: String::new()
        }]
    );
}

#[test]
fn a_wrong_version_fails_validation_with_both_versions() {
    let mut bad = request(vec![step(1, RootAction::RefreshKeyring)]);
    bad.version = 0;

    assert_eq!(
        bad.validate().unwrap_err(),
        RequestError::VersionMismatch {
            expected: PROTOCOL_VERSION,
            got: 0,
        }
    );
}

#[test]
fn a_terminal_size_comes_from_its_columns_and_rows() {
    assert_eq!(
        TerminalSize::from((80_u16, 24_u16)),
        TerminalSize {
            columns: 80,
            rows: 24,
        }
    );
}
