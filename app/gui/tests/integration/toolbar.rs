//! The toolbar row under the page header looks the same on every page.

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui::Setup;

use crate::common::launch;

/// The width of the sort dropdown named `sort` on the page behind the
/// sidebar entry `page`.
fn sort_width(page: &str, sort: &str) -> f32 {
    let mut harness = launch(Setup::default());

    harness.get_by_role_and_label(Role::Tab, page).click();
    harness.run();

    harness.get_by_label(sort).rect().width()
}

#[test]
fn every_sort_dropdown_has_one_width() {
    // The services' sort shows once a list exists, so the units are read first.
    let (harness, _runtime) = crate::services::services();
    let services = harness.get_by_label("Sort services").rect().width();
    let catalog = sort_width("App Catalog", "Sort apps");

    assert!(
        (services - catalog).abs() < 0.5,
        "Services {services} against App Catalog {catalog}"
    );
}
