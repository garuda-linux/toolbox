//! The scroll bars a page paints: a body taller than its window shows a
//! handle, which the egui bars alone never do.

use toolbox_gui_kit::testing::scroll_handles;

use crate::maintenance::maintenance;

#[test]
fn an_overflowing_page_body_paints_its_scroll_handle() {
    let mut fixture = maintenance();
    let harness = fixture.harness();
    harness.run();

    let handles = scroll_handles(harness);
    assert!(!handles.is_empty(), "no scroll handle in the page body");
}
