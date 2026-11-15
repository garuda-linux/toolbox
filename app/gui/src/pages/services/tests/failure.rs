use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_plan::UnitScope;

use super::{ServicesPage, listed, sshd};
use crate::command::{Command, ServiceCommand};
use crate::page_service::Services;
use crate::testing::{PageFixture, failure_read_again_sends, rect_of};
use crate::units::{UnitListing, UnitLists, Units, UnitsFailure};

const fn refused() -> UnitListing {
    UnitListing::Failed {
        error: UnitsFailure::Denied,
        previous: None,
    }
}

#[test]
fn a_refused_list_says_why_and_offers_to_read_again() {
    let mut harness = listed(refused()).harness(ServicesPage::default());

    harness.run();
    assert!(
        harness
            .query_by_label("You are not allowed to read the services.")
            .is_some()
    );

    assert_eq!(
        failure_read_again_sends(listed(refused()), ServicesPage::default()),
        [Command::from(ServiceCommand::Refresh(UnitScope::System))]
    );
}

#[test]
fn a_failed_relist_keeps_the_units_that_were_read() {
    let failed = UnitListing::Failed {
        error: UnitsFailure::Failed,
        previous: Some(Arc::new(vec![sshd()])),
    };

    let mut harness = listed(failed).harness(ServicesPage::default());
    harness.run();

    assert!(harness.query_by_label_contains("OpenSSH Daemon").is_some());
    assert!(
        harness
            .query_by_label_contains("The services could not be read.")
            .is_some()
    );

    // The header has its own Read again, the banner one sits below it.
    let banner = harness
        .query_all_by_role_and_label(Role::Button, "Read again")
        .max_by(|one, other| one.rect().top().total_cmp(&other.rect().top()))
        .expect("a Read again button");

    assert!(banner.rect().top() > rect_of(&harness, Role::TextInput, "Search services").top());
    banner.click();
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(ServiceCommand::Refresh(UnitScope::System))]
    );
}

/// Only the user manager failed: the System tab's systemd is fine.
#[test]
fn a_user_list_that_failed_names_the_user_manager() {
    let fixture = PageFixture {
        services: Services {
            units: Units::showing(UnitLists {
                system: UnitListing::Loaded(Arc::new(vec![sshd()])),
                user: UnitListing::Failed {
                    error: UnitsFailure::Unavailable,
                    previous: None,
                },
            }),
            ..Services::default()
        },
        ..PageFixture::default()
    };

    let mut page = ServicesPage::default();
    page.show_scope(UnitScope::User);
    let mut harness = fixture.harness(page);
    harness.run();

    assert!(
        harness
            .query_by_label("Your user's systemd is not reachable.")
            .is_some()
    );
    assert!(
        harness
            .query_by_label("systemd is not reachable.")
            .is_none()
    );
}

/// Chips, search and sort narrow a list, so they show off until one exists.
#[test]
fn without_a_list_no_chip_search_or_sort_is_on() {
    for listing in [UnitListing::NotLoaded, refused()] {
        let mut harness = listed(listing).harness(ServicesPage::default());
        harness.run_steps(3);

        let search = harness.get_by_label("Search services");
        assert!(search.accesskit_node().is_disabled());
        let sort = harness.get_by_label("Sort services");
        assert!(sort.accesskit_node().is_disabled());
        let running = harness
            .query_by_label_contains("Running")
            .expect("the Running chip");

        assert!(running.accesskit_node().is_disabled());
    }

    let mut harness =
        listed(UnitListing::Loaded(Arc::new(vec![sshd()]))).harness(ServicesPage::default());

    harness.run();
    assert!(harness.query_by_label("Search services").is_some());
}
