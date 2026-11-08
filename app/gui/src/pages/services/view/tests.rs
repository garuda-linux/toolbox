use toolbox_core::domains::{SystemUnitMasks, SystemUnits, UserUnits};
use toolbox_core::features::services::{ActiveState, FileState, Filter, UnitInfo, UnitOrder};
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_plan::{UnitName, UnitScope};

use super::rows;

fn unit(name: &str, active: ActiveState, file_state: FileState) -> UnitInfo {
    UnitInfo {
        name: UnitName::new(name).unwrap(),
        description: format!("About {name}"),
        active,
        sub_state: String::new(),
        file_state,
    }
}

fn units() -> Vec<UnitInfo> {
    vec![
        unit("sshd.service", ActiveState::Active, FileState::Enabled),
        unit("bluetooth.service", ActiveState::Failed, FileState::Enabled),
        unit("cups.service", ActiveState::Inactive, FileState::Disabled),
    ]
}

fn everything() -> Filter {
    Filter {
        search: String::new(),
        include_disabled: true,
        hide_inactive: false,
    }
}

fn names<'units>(rows: &[super::Row<'units>]) -> Vec<&'units str> {
    rows.iter().map(|row| row.unit.name.as_str()).collect()
}

#[test]
fn the_filter_and_order_pick_the_rows() {
    let units = units();
    let changes = ChangeSet::default();

    let running = rows(
        &units,
        &Filter::default(),
        UnitOrder::Name,
        UnitScope::System,
        &changes,
    );

    assert_eq!(names(&running), ["bluetooth.service", "sshd.service"]);

    let by_state = rows(
        &units,
        &everything(),
        UnitOrder::State,
        UnitScope::System,
        &changes,
    );

    assert_eq!(
        names(&by_state),
        ["bluetooth.service", "sshd.service", "cups.service"]
    );
}

#[test]
fn a_queued_wish_shows_what_the_unit_will_be() {
    let units = units();
    let snapshot = Snapshot::default();
    let mut changes = ChangeSet::default();
    changes.want::<SystemUnits>(
        UnitName::new("cups.service").unwrap(),
        Toggle::On,
        &snapshot,
    );
    changes.want::<SystemUnitMasks>(
        UnitName::new("sshd.service").unwrap(),
        Toggle::On,
        &snapshot,
    );

    let listed = rows(
        &units,
        &everything(),
        UnitOrder::Name,
        UnitScope::System,
        &changes,
    );

    let cups = listed
        .iter()
        .find(|row| row.unit.name.as_str() == "cups.service")
        .unwrap();

    let sshd = listed
        .iter()
        .find(|row| row.unit.name.as_str() == "sshd.service")
        .unwrap();

    assert_eq!((cups.enable, cups.enabled), (Some(Toggle::On), true));
    assert_eq!((sshd.mask, sshd.masked), (Some(Toggle::On), true));
    assert!(sshd.enabled, "nothing queued keeps the unit file's state");
}

#[test]
fn the_scope_picks_whose_wishes_count() {
    let units = units();
    let snapshot = Snapshot::default();
    let mut changes = ChangeSet::default();
    changes.want::<UserUnits>(
        UnitName::new("cups.service").unwrap(),
        Toggle::On,
        &snapshot,
    );

    let system = rows(
        &units,
        &everything(),
        UnitOrder::Name,
        UnitScope::System,
        &changes,
    );

    let user = rows(
        &units,
        &everything(),
        UnitOrder::Name,
        UnitScope::User,
        &changes,
    );

    assert!(system.iter().all(|row| row.enable.is_none()));
    assert!(user.iter().any(|row| row.enable == Some(Toggle::On)));
}
