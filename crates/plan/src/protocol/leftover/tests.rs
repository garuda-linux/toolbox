use super::Leftover;
use crate::protocol::{Failure, Outcome, decode, encode};
use crate::repair::{EspPlace, RepairMount};

#[test]
fn a_failed_step_names_what_its_cleanup_left() {
    let outcome = Outcome::Failed {
        cause: Failure::ExitCode { code: 32_i32 },
        cleanup_failed: vec![
            Leftover::Mounted {
                place: RepairMount::Esp(EspPlace::BootEfi),
            },
            Leftover::Open,
        ],
    };

    let line = serde_json::to_string(&outcome).unwrap();

    assert_eq!(serde_json::from_str::<Outcome>(&line).unwrap(), outcome);
}

/// A failure without leftovers keeps the line it always had.
#[test]
fn a_failure_without_leftovers_keeps_its_old_line() {
    let line = r#"{"result":"failed","cause":{"kind":"unknown"}}"#;
    let outcome = Outcome::Failed {
        cause: Failure::Unknown,
        cleanup_failed: Vec::new(),
    };

    assert_eq!(serde_json::to_string(&outcome).unwrap(), line);
    assert_eq!(serde_json::from_str::<Outcome>(line).unwrap(), outcome);
}

#[test]
fn a_leftover_says_where() {
    let root = Leftover::Mounted {
        place: RepairMount::Root,
    };

    assert!(root.to_string().ends_with("is still mounted"), "{root}");
    assert!(Leftover::Open.to_string().starts_with("/dev/mapper/"));
}

#[test]
fn a_failure_names_exactly_one_cause_on_the_wire() {
    let outcome = |cause| {
        encode(&Outcome::Failed {
            cause,
            cleanup_failed: Vec::new(),
        })
        .unwrap()
    };

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
            Outcome::Failed {
                cause,
                cleanup_failed: Vec::new()
            }
        );
    }

    assert!(
        decode::<Outcome>(r#"{"result":"failed","code":1,"signal":9}"#).is_err(),
        "a cause with both a code and a signal"
    );
}
