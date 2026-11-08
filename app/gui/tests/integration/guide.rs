use egui::accesskit::Role;
use egui::{Key, Theme};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui::page::PageId;
use toolbox_gui::{App, Setup};

use crate::common::{HEIGHT, launch_in, window};

/// Wide enough for the page list, the text and "On this page".
const WIDE: egui::Vec2 = egui::vec2(1440.0, 900.0);

fn app_on(page: PageId) -> Harness<'static, App> {
    let setup = Setup {
        open: Some(page),
        ..Setup::default()
    };

    launch_in(window(HEIGHT).with_theme(Theme::Dark), setup)
}

#[test]
fn guide_page() {
    let mut harness = app_on(PageId::Kernels);

    harness.key_press(Key::F1);
    harness.run();

    harness.snapshot("guide/page_dark");
}

#[test]
fn guide_page_wide_with_a_table_and_a_warning() {
    let mut harness = app_on(PageId::Kernels);

    harness.set_size(WIDE);
    harness.key_press(Key::F1);
    harness.run();
    harness
        .get_by_role_and_label(Role::Link, "What the list shows")
        .click();
    harness.run();
    harness.run();

    harness.snapshot("guide/wide_dark");
}

#[test]
fn guide_page_cuts_a_long_heading_in_its_outline() {
    let mut harness = app_on(PageId::Guide);

    harness.set_size(WIDE);
    harness.run();
    harness
        .query_all_by_role_and_label(Role::Link, "Command palette and shortcuts")
        .next()
        .expect("the list names the page")
        .click();
    harness.run();

    harness.snapshot("guide/outline_cut_dark");
}
