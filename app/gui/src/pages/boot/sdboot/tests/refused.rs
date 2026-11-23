use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_services::boot_menu::Menu;
use toolbox_system::MenuRead;
use toolbox_system::pkexec::Refusal;

use super::{sent, shown};
use crate::command::{BootCommand, Command};
use crate::testing::lowest_read_again;

fn refused(refusal: Refusal) -> Menu {
    Menu::Read(MenuRead::Refused(refusal))
}

/// Without a polkit agent or the helper, a second read is refused the
/// same way, so only the header's Read again stays.
#[test]
fn a_menu_no_read_can_reach_says_why_without_read_again() {
    for (refusal, said) in [
        (Refusal::NoAgent, "no polkit authentication agent"),
        (Refusal::HelperMissing, "Reinstall Garuda Toolbox"),
    ] {
        let harness = shown(refused(refusal));

        assert!(
            harness.query_by_label_contains(said).is_some(),
            "{refusal:?}"
        );
        assert_eq!(
            harness
                .query_all_by_role_and_label(Role::Button, "Read Again")
                .count(),
            1,
            "{refusal:?}: only the header's"
        );
    }
}

#[test]
fn a_closed_password_dialog_offers_to_read_the_menu_again() {
    let mut harness = shown(refused(Refusal::Denied));

    lowest_read_again(&harness).click();
    harness.run();

    assert_eq!(sent(&harness), [Command::from(BootCommand::Refresh)]);
}

#[test]
fn a_pkexec_that_cannot_run_says_so() {
    let harness = shown(refused(Refusal::Broken));

    assert!(
        harness
            .query_by_label_contains("pkexec cannot run on this system")
            .is_some()
    );
}
