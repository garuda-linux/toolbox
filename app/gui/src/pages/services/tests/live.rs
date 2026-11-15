use egui::accesskit::{Live, Role};
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::features::services::{ActiveState, FileState, Shown, UnitInfo};

use super::{SSHD, ServicesPage, loaded, sshd, unit};

fn spoken_live(harness: &egui_kittest::Harness<'static, super::PageState>) -> Live {
    harness.get_by_label("Running").accesskit_node().live()
}

#[test]
fn a_failed_unit_is_read_out_at_once() {
    let failed = UnitInfo {
        active: ActiveState::Failed,
        sub_state: "failed".to_owned(),
        ..unit("bluetooth.service", "Bluetooth", FileState::Enabled)
    };

    let mut harness = loaded(vec![failed]).harness(ServicesPage::default());
    harness.state_mut().page.filter.shown = Shown::Failed;
    harness.run();

    assert!(harness.query_by_role(Role::Alert).is_some());
}

#[test]
fn only_the_focused_unit_tells_its_state_changes() {
    let mut harness = loaded(vec![sshd()]).harness(ServicesPage::default());
    harness.run();

    assert_eq!(
        spoken_live(&harness),
        Live::Off,
        "a row away from the keyboard"
    );

    harness.get_by_role_and_label(Role::ListItem, SSHD).focus();
    harness.run();

    assert_eq!(spoken_live(&harness), Live::Polite);
}
