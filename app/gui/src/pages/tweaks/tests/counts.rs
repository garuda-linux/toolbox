//! The counts of the Shell and network section, on its heading and its
//! chip, and the pickers before the system was read.

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::Snapshot;

use super::{Shown, TweaksPage, shipped};
use crate::testing::PageFixture;

const PICKERS: [&str; 3] = ["Login shell", "DNS server", "Wi-Fi backend"];

fn unread() -> Shown {
    let mut harness = PageFixture {
        probed: false,
        snapshot: Snapshot::default().into(),
        ..shipped()
    }
    .harness(TweaksPage::default());

    harness.run();
    harness
}

/// The text beside the Shell and network heading, if any.
fn heading_count(harness: &Shown) -> Option<String> {
    let heading = harness
        .get_by_role_and_label(Role::Heading, "Shell and Network")
        .rect();

    harness
        .query_all_by_label_contains("/")
        .filter(|label| {
            let rect = label.rect();
            rect.left() > heading.right()
                && (rect.center().y - heading.center().y).abs() < heading.height()
        })
        .find_map(|label| {
            let node = label.accesskit_node();
            node.label().or_else(|| node.value())
        })
}

fn chip_label(harness: &Shown) -> String {
    harness
        .query_all_by_role(Role::Button)
        .filter_map(|button| button.accesskit_node().label())
        .find(|label| label.starts_with("Shell and Network"))
        .expect("the section has a chip")
}

#[test]
fn before_the_first_probe_the_pickers_rest_and_say_their_state_is_not_known() {
    let harness = unread();

    for picker in PICKERS {
        let node = harness.get_by_role_and_label(Role::ComboBox, picker);
        let node = node.accesskit_node();

        assert!(node.is_disabled(), "{picker} takes clicks");
        assert_eq!(node.value().as_deref(), Some("Not known"), "{picker}");
    }
}

#[test]
fn an_unread_switch_counts_as_unknown_on_its_heading_and_chip() {
    let harness = unread();

    assert_eq!(heading_count(&harness).as_deref(), Some("?/1"));
    assert!(
        chip_label(&harness).contains("?/1"),
        "{}",
        chip_label(&harness)
    );
}

/// A search that keeps the pickers but not the switch leaves nothing to
/// count, so neither the heading nor the chip shows "0/0".
#[test]
fn a_section_without_a_switch_in_view_has_no_count() {
    let mut harness = shipped().harness(TweaksPage::default());
    harness.run();
    harness.get_by_label("Search this page").click();
    harness.run();
    harness.get_by_label("Search this page").type_text("DNS");
    harness.run();

    assert_eq!(heading_count(&harness), None);
    assert!(
        !chip_label(&harness).contains('/'),
        "{}",
        chip_label(&harness)
    );
}
