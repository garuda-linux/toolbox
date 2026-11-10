use toolbox_plan::{PlannedAction, RootAction, TargetList, UnitOperation, UnitScope, UserAction};

mod critical;
mod queue;

use super::{
    ActiveState, FileState, Filter, UnitInfo, UnitOrder, does_nothing, pick, plan_control,
    plan_logs,
};
use crate::test_support::unit;

fn listed<'units>(
    units: &'units [UnitInfo],
    filter: &Filter,
    order: UnitOrder,
) -> Vec<&'units UnitInfo> {
    pick(units, filter, order)
        .into_iter()
        .map(|at| &units[at])
        .collect()
}

fn info(name: &str, description: &str, active: ActiveState, file_state: FileState) -> UnitInfo {
    UnitInfo {
        name: unit(name),
        description: description.to_owned(),
        active,
        sub_state: "running".to_owned(),
        file_state,
    }
}

fn units() -> Vec<UnitInfo> {
    vec![
        info(
            "sshd.service",
            "OpenSSH Daemon",
            ActiveState::Active,
            FileState::Enabled,
        ),
        info(
            "cups.socket",
            "CUPS Scheduler",
            ActiveState::Inactive,
            FileState::Disabled,
        ),
        info(
            "bluetooth.service",
            "Bluetooth service",
            ActiveState::Failed,
            FileState::Enabled,
        ),
        info(
            "avahi-daemon.service",
            "Avahi mDNS/DNS-SD Stack",
            ActiveState::Active,
            FileState::Disabled,
        ),
        info(
            "systemd-journald.service",
            "Journal Service",
            ActiveState::Active,
            FileState::Static,
        ),
    ]
}

fn names(units: &[&UnitInfo]) -> Vec<String> {
    units.iter().map(|unit| unit.name.to_string()).collect()
}

fn everything() -> Filter {
    Filter {
        search: String::new(),
        include_disabled: true,
        hide_inactive: false,
    }
}

#[test]
fn active_states_parse_from_systemd_strings() {
    let states = [
        ("active", ActiveState::Active),
        ("reloading", ActiveState::Reloading),
        ("inactive", ActiveState::Inactive),
        ("failed", ActiveState::Failed),
        ("activating", ActiveState::Activating),
        ("deactivating", ActiveState::Deactivating),
        ("maintenance", ActiveState::Maintenance),
        ("refreshing", ActiveState::Refreshing),
        ("frobnicating", ActiveState::Other),
        ("", ActiveState::Other),
    ];

    for (text, state) in states {
        assert_eq!(ActiveState::parse(text), state, "{text}");
    }

    assert_eq!(ActiveState::Deactivating.as_str(), "deactivating");
    assert_eq!(ActiveState::Other.as_str(), "other");
}

#[test]
fn file_states_fold_runtime_and_link_variants() {
    let states = [
        ("enabled", FileState::Enabled),
        ("enabled-runtime", FileState::Enabled),
        ("disabled", FileState::Disabled),
        ("static", FileState::Static),
        ("masked", FileState::Masked),
        ("masked-runtime", FileState::Masked),
        ("indirect", FileState::Indirect),
        ("generated", FileState::Generated),
        ("linked", FileState::Linked),
        ("linked-runtime", FileState::Linked),
        ("alias", FileState::Alias),
        ("transient", FileState::Transient),
        ("bad", FileState::Bad),
        ("enabled-forever", FileState::Other),
    ];

    for (text, state) in states {
        assert_eq!(FileState::parse(text), state, "{text}");
    }

    assert_eq!(FileState::Masked.as_str(), "masked");
    assert_eq!(FileState::Unknown.as_str(), "unknown");
    assert_eq!(FileState::Linked.as_str(), "linked");
    assert_eq!(FileState::Transient.as_str(), "transient");
}

#[test]
fn the_default_filter_shows_what_runs() {
    assert_eq!(
        Filter::default(),
        Filter {
            search: String::new(),
            include_disabled: false,
            hide_inactive: true,
        }
    );
}

#[test]
fn everything_shows_sorted_by_name() {
    let units = units();

    assert_eq!(
        names(&listed(&units, &everything(), UnitOrder::Name)),
        [
            "avahi-daemon.service",
            "bluetooth.service",
            "cups.socket",
            "sshd.service",
            "systemd-journald.service"
        ]
    );
}

#[test]
fn the_search_matches_name_or_description_ignoring_case() {
    let units = units();
    let by_name = Filter {
        search: "SSH".to_owned(),
        ..everything()
    };

    let by_description = Filter {
        search: "dns-sd".to_owned(),
        ..everything()
    };

    assert_eq!(
        names(&listed(&units, &by_name, UnitOrder::Name)),
        ["sshd.service"]
    );
    assert_eq!(
        names(&listed(&units, &by_description, UnitOrder::Name)),
        ["avahi-daemon.service"]
    );
}

#[test]
fn surrounding_spaces_in_the_search_are_ignored() {
    let units = units();
    let filter = Filter {
        search: "  cups ".to_owned(),
        ..everything()
    };

    assert_eq!(
        names(&listed(&units, &filter, UnitOrder::Name)),
        ["cups.socket"]
    );
}

#[test]
fn disabled_units_can_be_left_out() {
    let units = units();
    let filter = Filter {
        include_disabled: false,
        ..everything()
    };

    assert_eq!(
        names(&listed(&units, &filter, UnitOrder::Name)),
        [
            "bluetooth.service",
            "sshd.service",
            "systemd-journald.service"
        ]
    );
}

/// A failed unit is what the user most likely came for, so hiding the
/// inactive ones keeps it.
#[test]
fn hiding_inactive_units_keeps_failed_ones() {
    let units = units();
    let filter = Filter {
        hide_inactive: true,
        ..everything()
    };

    assert_eq!(
        names(&listed(&units, &filter, UnitOrder::Name)),
        [
            "avahi-daemon.service",
            "bluetooth.service",
            "sshd.service",
            "systemd-journald.service"
        ]
    );
}

#[test]
fn system_operations_run_through_the_helper() {
    let plan = plan_control(
        UnitOperation::Restart,
        UnitScope::System,
        unit("sshd.service"),
    );

    let actions: Vec<&PlannedAction> = plan.steps().iter().map(|step| &step.action).collect();

    assert_eq!(
        actions,
        [&PlannedAction::Root(RootAction::ControlUnits {
            operation: UnitOperation::Restart,
            units: TargetList::one(unit("sshd.service")),
        })]
    );
}

#[test]
fn user_operations_run_as_the_user() {
    let plan = plan_control(UnitOperation::Stop, UnitScope::User, unit("psd.service"));
    let actions: Vec<&PlannedAction> = plan.steps().iter().map(|step| &step.action).collect();

    assert_eq!(
        actions,
        [&PlannedAction::User(UserAction::ControlUserUnits {
            operation: UnitOperation::Stop,
            units: TargetList::one(unit("psd.service")),
        })]
    );
}

#[test]
fn logs_are_read_as_the_user_for_both_scopes() {
    for scope in [UnitScope::System, UnitScope::User] {
        let plan = plan_logs(scope, unit("sshd.service"));
        let actions: Vec<&PlannedAction> = plan.steps().iter().map(|step| &step.action).collect();

        assert_eq!(
            actions,
            [&PlannedAction::User(UserAction::UnitLogs {
                unit: unit("sshd.service"),
                scope,
            })]
        );
    }
}

#[test]
fn the_list_is_sorted_ignoring_case() {
    let units = [
        info(
            "NetworkManager.service",
            "",
            ActiveState::Active,
            FileState::Enabled,
        ),
        info("acpid.service", "", ActiveState::Active, FileState::Enabled),
    ];

    assert_eq!(
        names(&listed(&units, &everything(), UnitOrder::Name)),
        ["acpid.service", "NetworkManager.service"]
    );
}

#[test]
fn by_state_failed_units_lead_and_stopped_ones_trail() {
    let mut units = units();

    if let Some(journal) = units.last_mut() {
        "exited".clone_into(&mut journal.sub_state);
    }

    units.push(info(
        "upower.service",
        "Power Management",
        ActiveState::Activating,
        FileState::Enabled,
    ));

    assert_eq!(
        names(&listed(&units, &everything(), UnitOrder::Name)),
        [
            "avahi-daemon.service",
            "bluetooth.service",
            "cups.socket",
            "sshd.service",
            "systemd-journald.service",
            "upower.service",
        ]
    );

    assert_eq!(
        names(&listed(&units, &everything(), UnitOrder::State)),
        [
            "bluetooth.service",
            "upower.service",
            "systemd-journald.service",
            "avahi-daemon.service",
            "sshd.service",
            "cups.socket",
        ],
        "failed, changing, running by sub state then name, stopped"
    );
}

#[test]
fn start_does_nothing_on_a_running_unit_and_stop_nothing_on_a_stopped_one() {
    for running in [
        ActiveState::Active,
        ActiveState::Reloading,
        ActiveState::Activating,
    ] {
        assert!(does_nothing(UnitOperation::Start, running), "{running:?}");
        assert!(!does_nothing(UnitOperation::Stop, running), "{running:?}");
    }

    for stopped in [ActiveState::Inactive, ActiveState::Failed] {
        assert!(does_nothing(UnitOperation::Stop, stopped), "{stopped:?}");
        assert!(!does_nothing(UnitOperation::Start, stopped), "{stopped:?}");
    }

    for operation in [UnitOperation::Restart, UnitOperation::Reload] {
        assert!(!does_nothing(operation, ActiveState::Active));
    }
}
