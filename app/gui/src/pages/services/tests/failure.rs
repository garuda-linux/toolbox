use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_plan::UnitScope;

use super::{ServicesPage, listed, sshd};
use crate::command::{Command, ServiceCommand};
use crate::testing::rect_of;
use crate::units::{UnitListing, UnitsFailure};

#[test]
fn a_refused_list_says_why_and_offers_a_retry() {
    let failed = UnitListing::Failed {
        error: UnitsFailure::Denied,
        previous: None,
    };

    let mut harness = listed(failed).harness(ServicesPage::default());

    harness.run();
    assert!(
        harness
            .query_by_label("You are not allowed to read the services.")
            .is_some()
    );

    harness.get_by_role_and_label(Role::Button, "Retry").click();
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
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
