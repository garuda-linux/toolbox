//! The command palette finds repository and AUR packages, also before
//! the Packages page was opened, and opens a package's details.

use std::sync::Arc;

use egui::accesskit::Role;
use egui::{Key, Theme};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_gui::App;
use toolbox_gui::packages::SampleAur;

use crate::common::palette;
use crate::packages::launch;
use crate::system::step_until;

fn row(harness: &Harness<'static, App>, name: &str) -> bool {
    harness
        .query_all_by_role(Role::Button)
        .filter_map(|node| node.accesskit_node().label())
        .any(|label| label.starts_with(name))
}

fn found(harness: &mut Harness<'static, App>, text: &str, name: &str) {
    palette(harness, text);
    step_until(harness, "the package in the palette", |harness| {
        row(harness, name)
    });
}

#[test]
fn a_package_is_found_before_its_page_opened_and_opens_its_details() {
    let (mut harness, _runtime) = launch(Theme::Dark, None, false, Arc::new(SampleAur));

    found(&mut harness, "gegl", "gegl");

    harness.key_press(Key::Enter);
    step_until(&mut harness, "the details", |harness| {
        harness.query_by_label("https://www.gegl.org/").is_some()
    });
}

#[test]
fn an_aur_package_is_found_only_while_the_aur_is_on() {
    let (mut off, _off_runtime) = launch(Theme::Dark, None, false, Arc::new(SampleAur));
    palette(&mut off, "yay-bin");
    off.run();
    assert!(!row(&off, "yay-bin"));

    let (mut on, _on_runtime) = launch(Theme::Dark, None, true, Arc::new(SampleAur));
    found(&mut on, "yay-bin", "yay-bin");

    on.key_press(Key::Enter);
    step_until(&mut on, "the AUR details", |harness| {
        harness
            .query_by_label("https://aur.archlinux.org/packages/yay-bin")
            .is_some()
    });
}

#[test]
fn more_than_five_packages_end_with_a_row_that_lists_them_all() {
    let (mut harness, _runtime) = launch(Theme::Dark, None, false, Arc::new(SampleAur));

    let packages_row = |node: &egui_kittest::Node<'_>| {
        node.accesskit_node()
            .label()
            .is_some_and(|label| label.starts_with("Show All") && label.ends_with("in Packages"))
    };

    // Other sections show their own Show All row before the package search lands.
    palette(&mut harness, "a");
    step_until(&mut harness, "the Packages row", |harness| {
        harness
            .query_all_by_role(Role::Button)
            .any(|node| packages_row(&node))
    });
    harness
        .query_all_by_role(Role::Button)
        .find(packages_row)
        .expect("the row")
        .click_accesskit();

    step_until(&mut harness, "the searched list", |harness| {
        harness
            .query_by_role_and_label(Role::TextInput, "Search packages")
            .is_some()
    });
}
