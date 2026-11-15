use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::Snapshot;
use toolbox_services::boot_menu::Menu;
use toolbox_system::pkexec::Refusal;
use toolbox_system::{FailedProbes, Host, MenuRead, Probe};

use super::super::BootPage;
use super::{grub_system, page};
use crate::boot::Boot;
use crate::command::{BootCommand, Command};
use crate::testing::{PageFixture, failure_read_again_sends, lowest_read_again};

#[test]
fn before_the_first_read_placeholder_rows_wait() {
    let harness = page(Snapshot::default(), Boot::default());

    assert!(
        harness
            .query_by_role_and_label(Role::ProgressIndicator, "Reading the boot settings…")
            .is_some()
    );
}

#[test]
fn grub_settings_that_could_not_be_read_offer_to_read_again() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(Host::default());
    snapshot.insert(FailedProbes::new([Probe::Grub]));
    let fixture = PageFixture {
        snapshot: Arc::new(snapshot),
        ..PageFixture::default()
    };

    assert_eq!(
        failure_read_again_sends(fixture, BootPage::default()),
        [Command::from(BootCommand::Refresh)]
    );
}

fn grub_menu(read: MenuRead) -> super::Shown {
    page(grub_system(), Boot::showing(Menu::Read(read), None, None))
}

/// The file is read, only the menu is not: one banner over the card with
/// its own Read again, the default entry row keeps its description.
#[test]
fn a_grub_menu_that_could_not_be_read_says_so_in_a_banner() {
    let mut harness = grub_menu(MenuRead::Refused(Refusal::Denied));

    assert!(
        harness
            .query_by_label_contains("the password dialog was closed")
            .is_some()
    );
    assert!(
        harness
            .query_by_label("The menu entry that is booted when you do not pick another.")
            .is_some()
    );

    lowest_read_again(&harness).click();
    harness.run();
    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(BootCommand::Refresh)]
    );
}

#[test]
fn a_grub_menu_no_read_can_reach_has_no_second_read_again() {
    let harness = grub_menu(MenuRead::Refused(Refusal::NoAgent));

    assert!(
        harness
            .query_by_label_contains("no polkit authentication agent")
            .is_some()
    );
    assert_eq!(
        harness
            .query_all_by_role_and_label(Role::Button, "Read again")
            .count(),
        1
    );
}

#[test]
fn a_grub_menu_pkexec_cannot_read_says_so() {
    let harness = grub_menu(MenuRead::Refused(Refusal::Broken));

    assert!(
        harness
            .query_by_label_contains("pkexec cannot run on this system")
            .is_some()
    );
}
