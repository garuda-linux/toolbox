use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::features::packages::list::Filter;
use toolbox_gui_kit::Loadable;
use toolbox_plan::PackageName;
use toolbox_system::{PendingUpdate, ProbeError, Status};

use super::super::PackagesPage;
use super::{fixture, listed};
use crate::command::{Command, PackagesCommand};

fn updates_page() -> PackagesPage {
    PackagesPage {
        filter: Filter::Updates,
        ..PackagesPage::default()
    }
}

fn status(updates: Result<Vec<PendingUpdate>, ProbeError>) -> Status {
    Status {
        updates,
        aur_updates: Err(ProbeError::Unavailable),
        merges: Ok(Vec::new()),
        last_update: Err(ProbeError::Unavailable),
        health: Ok(Vec::new()),
    }
}

#[test]
fn before_the_status_is_read_the_updates_chip_has_no_count_and_a_loading_line() {
    let mut harness = fixture(None).harness(updates_page());
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Updates")
            .is_some()
    );
    assert!(harness.query_by_label_contains("Updates 0").is_none());
    assert!(
        harness
            .query_by_label_contains("Checking for updates")
            .is_some()
    );
    assert!(harness.query_by_label("No pending updates").is_none());
}

#[test]
fn a_failed_check_shows_a_banner_that_reads_again_and_no_count() {
    let mut fixture = fixture(None);
    fixture.status = Some(Loadable::Failed {
        error: "checkupdates stopped".to_owned(),
        previous: None,
    });

    let mut harness = fixture.harness(updates_page());
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Updates")
            .is_some()
    );
    assert!(harness.query_by_label("No pending updates").is_none());
    assert!(
        harness
            .query_by_label_contains("Updates could not be checked")
            .is_some()
    );

    // The header's Read again comes first.
    let banner = harness.query_all_by_label("Read again").last().unwrap();
    banner.click();
    harness.run();

    assert!(
        harness
            .state()
            .fixture
            .sent()
            .contains(&Command::from(PackagesCommand::Refresh))
    );
}

#[test]
fn a_failed_repository_check_inside_a_read_status_counts_as_failed() {
    let mut fixture = fixture(None);
    fixture.status = Some(Loadable::Loaded(Arc::new(status(Err(
        ProbeError::Unavailable,
    )))));

    let mut harness = fixture.harness(updates_page());
    harness.run();

    assert!(
        harness
            .query_by_label_contains("Updates could not be checked")
            .is_some()
    );
}

#[test]
fn a_read_status_counts_its_updates_on_the_chip() {
    let mut fixture = fixture(None);
    let update = PendingUpdate {
        name: PackageName::new("gimp").unwrap(),
        from: "1".to_owned(),
        to: "2".to_owned(),
    };

    fixture.status = Some(Loadable::Loaded(Arc::new(status(Ok(vec![update])))));

    let mut harness = fixture.harness(updates_page());
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Updates 1")
            .is_some()
    );
    assert!(listed(&harness, "gimp"));
    assert!(
        harness
            .query_by_label_contains("Updates could not be checked")
            .is_none()
    );
}
