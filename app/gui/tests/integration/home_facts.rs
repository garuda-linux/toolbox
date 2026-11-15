//! The stat tiles below the Home greeting and the status rows the facts
//! add.

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_plan::{PackageName, UnitName};
use toolbox_system::{DiskSpace, ProbeError, Status};

use crate::home::{GB, home, home_with, host, quiet, quiet_facts};

#[test]
fn the_disk_tile_names_the_free_space_and_opens_maintenance() {
    let (mut harness, _runtime, _power) = home_with(host(false, false), quiet(), quiet_facts());

    harness
        .get_by_role_and_label(Role::Link, "Disk: 186.3 GiB free")
        .click();
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Maintenance")
            .is_some()
    );
}

#[test]
fn a_low_disk_offers_maintenance() {
    let mut facts = quiet_facts();
    facts.disk = Ok(DiskSpace {
        total: 500 * GB,
        free: 9 * GB,
    });
    let (mut harness, _runtime, _power) = home_with(host(false, false), quiet(), facts);

    assert!(harness.query_by_label("Disk space is low").is_some());
    harness.get_by_label("Open Maintenance").click();
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Maintenance")
            .is_some()
    );
}

#[test]
fn a_failed_service_is_named_and_leads_to_services() {
    let mut facts = quiet_facts();
    facts.failed_units = Ok(vec![UnitName::new("smb.service").unwrap()]);
    let (mut harness, _runtime, _power) = home_with(host(false, false), quiet(), facts);

    assert!(harness.query_by_label("A service failed").is_some());
    assert!(harness.query_by_label("smb.service").is_some());
    harness.get_by_label("Show in Services").click();
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Services")
            .is_some()
    );
}

#[test]
fn orphans_queue_their_clean_up_and_take_it_back() {
    let mut facts = quiet_facts();
    facts.orphans = Ok(vec![PackageName::new("old-lib").unwrap()]);
    let (mut harness, _runtime, _power) = home_with(host(false, false), quiet(), facts);

    assert!(
        harness
            .query_by_label("Orphans and package cache")
            .is_some()
    );
    harness.get_by_label("Queue clean-up").click();
    harness.run();
    assert!(harness.query_by_label("Don't clean up").is_some());

    harness.get_by_label("Don't clean up").click();
    harness.run();
    assert!(harness.query_by_label("Queue clean-up").is_some());
}

#[test]
fn a_small_cache_without_orphans_is_no_row() {
    let (harness, _runtime, _power) = home_with(host(false, false), quiet(), quiet_facts());

    assert!(
        harness
            .query_by_label("Orphans and package cache")
            .is_none()
    );
    assert!(harness.query_by_label("Everything is fine").is_some());
}

#[test]
fn a_failed_check_is_said_once_by_its_banner() {
    let status = Status {
        health: Err(ProbeError::fake("no garuda-health")),
        ..quiet()
    };

    let (harness, _runtime, _power) = home(host(false, false), status);

    assert!(
        harness
            .query_by_label_contains("One check could not run")
            .is_some()
    );
    assert!(
        harness
            .query_by_label("Some checks could not run")
            .is_none()
    );
    assert!(harness.query_by_label("Everything is fine").is_none());
    assert!(
        harness
            .query_by_label("Maintain and set up your system.")
            .is_some()
    );
}
