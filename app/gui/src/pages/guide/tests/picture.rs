use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;

use super::page;

const ALT: &str = "The Config Files page with its file list";

/// The picture keeps its room from the first frame, so the words under
/// it never move while it is decoded.
#[test]
fn a_picture_keeps_its_room_before_it_is_shown() {
    let mut harness = page();
    harness.state_mut().page.show("config-files.md", None);
    for _ in 0_u8..5 {
        if harness.query_by_role_and_label(Role::Label, ALT).is_some() {
            break;
        }

        harness.step();
    }

    let first = harness.get_by_role_and_label(Role::Label, ALT).rect().top();
    harness.run();
    let settled = harness.get_by_role_and_label(Role::Label, ALT).rect().top();

    assert!(
        (first - settled).abs() < 0.5,
        "the words under the picture moved from {first} to {settled}"
    );
}
