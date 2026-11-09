use super::{UnitOperation, UnitScope};
use crate::action::testing::{shown, shown_user, units};
use crate::procedure::Invocation;
use crate::{RootAction, UnitName, UserAction};

const OPERATIONS: [UnitOperation; 4] = [
    UnitOperation::Start,
    UnitOperation::Stop,
    UnitOperation::Restart,
    UnitOperation::Reload,
];

#[test]
fn every_unit_operation_controls_system_units_through_systemctl() {
    for operation in OPERATIONS {
        let action = RootAction::ControlUnits {
            operation,
            units: units(&["sshd.service", "cups.socket"]),
        };

        assert_eq!(
            shown(&action),
            [format!(
                "systemctl {} -- cups.socket sshd.service",
                operation.as_str()
            )]
        );
    }
}

#[test]
fn unit_operations_have_stable_ids() {
    let ids: Vec<&str> = OPERATIONS.into_iter().map(UnitOperation::as_str).collect();

    assert_eq!(ids, ["start", "stop", "restart", "reload"]);
}

#[test]
fn control_units_are_kebab_case_on_the_wire() {
    let action = RootAction::ControlUnits {
        operation: UnitOperation::Restart,
        units: units(&["sshd.service"]),
    };

    let json = serde_json::to_string(&action).unwrap();
    assert_eq!(
        json,
        r#"{"kind":"control-units","operation":"restart","units":["sshd.service"]}"#
    );
    assert_eq!(serde_json::from_str::<RootAction>(&json).unwrap(), action);
    serde_json::from_str::<RootAction>(
        r#"{"kind":"control-units","operation":"kill","units":["sshd.service"]}"#,
    )
    .unwrap_err();
    serde_json::from_str::<RootAction>(
        r#"{"kind":"control-units","operation":"start","units":["--force"]}"#,
    )
    .unwrap_err();
}

#[test]
fn system_units_are_masked_and_unmasked_through_systemctl() {
    let mask = RootAction::MaskUnits {
        units: units(&["cups.service"]),
    };

    let unmask = RootAction::UnmaskUnits {
        units: units(&["cups.service", "cups.socket"]),
    };

    assert_eq!(shown(&mask), ["systemctl mask -- cups.service"]);
    assert_eq!(
        shown(&unmask),
        ["systemctl unmask -- cups.service cups.socket"]
    );
}

#[test]
fn user_units_are_controlled_through_the_user_manager() {
    let action = UserAction::ControlUserUnits {
        operation: UnitOperation::Stop,
        units: units(&["psd.service"]),
    };

    assert_eq!(
        shown_user(&action),
        ["systemctl --user stop -- psd.service"]
    );
}

/// systemctl rejects `--usermask`, so the argv keeps `--user` and `mask`
/// apart.
#[test]
fn masking_a_user_unit_keeps_user_and_mask_apart() {
    let argv = |action: &UserAction| -> Vec<Vec<String>> {
        action
            .procedure()
            .iter()
            .map(|invocation| match invocation {
                Invocation::Run(command) => command.arguments().to_vec(),
                other @ (Invocation::RunWithTargetsFrom { query: _, run: _ }
                | Invocation::Builtin(_)
                | Invocation::Always(_)
                | Invocation::Cleanup {
                    command: _,
                    leftover: _,
                }
                | Invocation::Feed {
                    command: _,
                    input: _,
                }) => {
                    panic!("not a program run: {other}")
                }
            })
            .collect()
    };

    let mask = UserAction::MaskUserUnits {
        units: units(&["psd.service"]),
    };

    let unmask = UserAction::UnmaskUserUnits {
        units: units(&["psd.service"]),
    };

    assert_eq!(argv(&mask), [["--user", "mask", "--", "psd.service"]]);
    assert_eq!(argv(&unmask), [["--user", "unmask", "--", "psd.service"]]);
}

#[test]
fn unit_logs_come_from_the_journal_as_the_user() {
    let system = UserAction::UnitLogs {
        unit: UnitName::new("sshd.service").unwrap(),
        scope: UnitScope::System,
    };

    let user = UserAction::UnitLogs {
        unit: UnitName::new("psd.service").unwrap(),
        scope: UnitScope::User,
    };

    assert_eq!(
        shown_user(&system),
        ["journalctl --no-pager -n 500 -u sshd.service"]
    );
    assert_eq!(
        shown_user(&user),
        ["journalctl --user --no-pager -n 500 -u psd.service"]
    );
}

#[test]
fn the_user_flag_picks_the_users_own_units() {
    assert_eq!(UnitScope::from_user(true), UnitScope::User);
    assert_eq!(UnitScope::from_user(false), UnitScope::System);
}

#[test]
fn unit_scopes_have_stable_ids() {
    assert_eq!(UnitScope::System.as_str(), "system");
    assert_eq!(UnitScope::User.as_str(), "user");
    assert_eq!(
        serde_json::to_string(&UnitScope::User).unwrap(),
        r#""user""#
    );
}
