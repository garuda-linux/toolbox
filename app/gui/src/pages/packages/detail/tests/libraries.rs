//! The sonames of a package's libraries wait behind one button after
//! its packages.

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;

use super::{gimp_with, showing};

#[test]
fn the_packages_show_first_and_the_libraries_behind_a_button() {
    let mut harness = showing(gimp_with(|info| {
        info.depends = ["libasound.so=2-64", "gegl>=0.4", "libavcodec.so=63-64"]
            .map(str::to_owned)
            .to_vec();
    }));

    assert!(
        harness
            .query_by_role_and_label(Role::Link, "gegl>=0.4")
            .is_some()
    );
    assert!(harness.query_by_label("libasound.so=2-64").is_none());

    harness
        .get_by_role_and_label(Role::Button, "Show 2 libraries")
        .click();
    harness.run();

    assert!(harness.query_by_label("libasound.so=2-64").is_some());
    assert!(harness.query_by_label("libavcodec.so=63-64").is_some());
    assert!(harness.query_by_label("Show 2 libraries").is_none());
}

#[test]
fn a_package_without_libraries_has_no_library_button() {
    let harness = showing(gimp_with(|_| {}));

    assert!(harness.query_by_label_contains("libraries").is_none());
}
