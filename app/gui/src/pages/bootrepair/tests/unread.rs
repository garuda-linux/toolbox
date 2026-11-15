use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::features::bootrepair::page::{Facts, Offer, Progress};

use super::super::BootRepairPage;
use super::page;
use crate::bootrepair::BootRepair;
use crate::command::{BootRepairCommand, Command};
use crate::page_service::Services;
use crate::testing::{PageFixture, failure_read_again_sends};

fn reading(facts: Facts) -> BootRepair {
    BootRepair::showing(Offer::Live, facts, Progress::None)
}

#[test]
fn while_the_disks_are_read_placeholder_rows_wait() {
    let harness = page(reading(Facts::Loading), BootRepairPage::default());

    assert!(
        harness
            .query_by_role_and_label(Role::ProgressIndicator, "Reading the disks…")
            .is_some()
    );
}

#[test]
fn disks_that_could_not_be_read_offer_to_read_them_again() {
    let fixture = PageFixture {
        services: Services {
            boot_repair: reading(Facts::Failed),
            ..Services::default()
        },
        ..PageFixture::default()
    };

    assert_eq!(
        failure_read_again_sends(fixture, BootRepairPage::default()),
        [Command::from(BootRepairCommand::Refresh)]
    );
}
