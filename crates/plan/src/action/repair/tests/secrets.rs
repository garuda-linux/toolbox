//! The passphrase of a locked system: fed to `cryptsetup`, and in no
//! text form of the action, its procedure or its request.

use super::{PASSPHRASE, bios, every_repair, fed, locked, open, repair, uefi};
use crate::procedure::Invocation;
use crate::protocol::{Mode, Request, encode};
use crate::{Password, RootAction, RootLayout, Step, StepId};

#[test]
fn the_passphrase_is_fed_and_never_an_argument() {
    let action = repair(
        locked("/dev/sda2", RootLayout::Subvolume),
        uefi("/dev/sda1"),
    );

    let procedure = action.procedure();
    let inputs: Vec<_> = procedure.iter().filter_map(fed).collect();

    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs.first().unwrap().expose(), PASSPHRASE.as_bytes());

    for invocation in &procedure {
        assert!(!invocation.to_string().contains("horse"), "{invocation}");
        assert!(
            !format!("{invocation:?}").contains("horse"),
            "{invocation:?}"
        );
    }
}

#[test]
fn no_text_form_of_a_repair_holds_the_passphrase() {
    let action = repair(
        locked("/dev/sda2", RootLayout::Subvolume),
        uefi("/dev/sda1"),
    );

    let request = Request::new(
        Mode::Run,
        vec![Step {
            id: StepId::new(0),
            action: action.clone(),
        }],
    );

    let line = encode(&request).unwrap();

    for text in [
        format!("{action:?}"),
        format!("{action:#?}"),
        format!("{request:?}"),
        serde_json::to_string(&action).unwrap(),
        line.clone(),
    ] {
        assert!(!text.contains("horse"), "{text}");
        assert!(!text.contains("staple"), "{text}");
    }

    assert!(!line.contains("passphrase"), "{line}");
}

#[test]
fn a_decoded_repair_waits_for_its_passphrase_on_its_own_line() {
    let action = repair(locked("/dev/sda2", RootLayout::Plain), bios("/dev/sda"));
    let mut decoded: RootAction =
        serde_json::from_str(&serde_json::to_string(&action).unwrap()).unwrap();

    assert!(decoded.needs_secret());
    assert_eq!(decoded.secret(), None);
    assert!(decoded.procedure().iter().any(|invocation| matches!(
        invocation,
        Invocation::Feed {
            command: _,
            input: None
        }
    )));

    assert!(decoded.attach_secret(Password::new(PASSPHRASE.to_owned()).unwrap()));
    assert_eq!(decoded, action);
}

#[test]
fn a_repair_of_an_open_root_takes_no_secret() {
    let mut action = repair(open("/dev/sda2", RootLayout::Plain), bios("/dev/sda"));

    assert!(!action.needs_secret());
    assert!(!action.attach_secret(Password::new(PASSPHRASE.to_owned()).unwrap()));
    assert_eq!(action.secret(), None);
}

#[test]
fn a_passphrase_field_in_the_request_is_ignored() {
    let line = serde_json::json!({
        "kind": "repair-boot",
        "system": {
            "root": { "kind": "locked", "partition": "/dev/sda2" },
            "layout": "subvolume",
        },
        "work": { "kind": "regenerate-config" },
        "passphrase": "smuggled",
    });

    let decoded: RootAction = serde_json::from_value(line).unwrap();

    assert_eq!(decoded.secret(), None);
}

#[test]
fn repairs_round_trip_through_json() {
    for action in every_repair() {
        let line = serde_json::to_string(&action).unwrap();
        let mut decoded: RootAction = serde_json::from_str(&line).unwrap();

        if let Some(secret) = action.secret() {
            assert!(decoded.attach_secret(secret.clone()));
        }

        assert_eq!(decoded, action, "{line}");
    }
}
