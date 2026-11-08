//! Where a page shows when the user comes to it: another page or tab
//! starts at the top, the same one keeps its place.

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui::App;

use crate::common::{open_tab, wheel};
use crate::maintenance::maintenance;

fn scroll_down(harness: &mut Harness<'static, App>) {
    wheel(harness, egui::pos2(600.0, 500.0), egui::vec2(0.0, -400.0));
    harness.run();
}

fn heading_top(harness: &Harness<'static, App>) -> f32 {
    harness.get_by_label("Package care").rect().top()
}

#[test]
fn another_tab_and_back_starts_at_the_top() {
    let mut fixture = maintenance();
    let harness = fixture.harness();
    let top = heading_top(harness);
    scroll_down(harness);
    assert!(heading_top(harness) < top - 50.0, "the page did not scroll");

    open_tab(harness, "Repairs");
    scroll_down(harness);
    open_tab(harness, "Common");
    assert!(
        (heading_top(harness) - top).abs() < 0.5,
        "Common kept its scroll after another tab"
    );
}

#[test]
fn a_refresh_keeps_the_place() {
    let mut fixture = maintenance();
    let harness = fixture.harness();
    scroll_down(harness);
    let scrolled = heading_top(harness);

    harness
        .get_by_role_and_label(Role::Button, "Read again")
        .click();
    harness.run();
    assert!((heading_top(harness) - scrolled).abs() < 0.5);
}

#[test]
fn another_page_and_back_starts_at_the_top() {
    let mut fixture = maintenance();
    let harness = fixture.harness();
    let top = heading_top(harness);
    scroll_down(harness);
    assert!(heading_top(harness) < top - 50.0, "the page did not scroll");

    for page in ["Home", "Maintenance"] {
        harness.get_by_role_and_label(Role::Tab, page).click();
        harness.run();
    }

    assert!(
        (heading_top(harness) - top).abs() < 0.5,
        "Maintenance kept its scroll after another page"
    );
}
