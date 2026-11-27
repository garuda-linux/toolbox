//! Every page at the smallest window and in light, with focus on its first control.

mod german;
mod idle;

use egui::{Key, Theme};
use egui_kittest::kittest::Queryable as _;
use egui_kittest::{Harness, SnapshotResults};
use toolbox_gui::page::PageId;
use toolbox_gui::{App, Setup};

use crate::captured::launch_covered;
use crate::common::{HEIGHT, SMALLEST, WIDTH};

const WIDE: (f32, f32) = (WIDTH, HEIGHT);

/// Shown only on the live ISO, so a page of its own elsewhere.
const SKIPPED: [PageId; 1] = [PageId::BootRepair];

fn page(page: PageId, theme: Theme, size: (f32, f32), setup: Setup) -> Harness<'static, App> {
    let setup = Setup {
        open: Some(page),
        catalogs: crate::discover::source_catalogs(),
        ..setup
    };

    launch_covered(
        toolbox_gui_kit::testing::harness(size.0, size.1).with_theme(theme),
        setup,
    )
}

fn pages() -> impl Iterator<Item = PageId> {
    PageId::ALL
        .into_iter()
        .filter(|page| !SKIPPED.contains(page))
}

/// Tabs on until a control right of the sidebar has the focus.
fn focus_page(harness: &mut Harness<'static, App>) {
    let sidebar = harness.ctx.content_rect().width() * 0.2;
    for _ in 0_u8..60 {
        harness.key_press(Key::Tab);
        harness.run();
        let focused = harness.ctx.memory(egui::Memory::focused);
        let right = focused
            .and_then(|id| harness.ctx.read_response(id))
            .is_some_and(|response| response.rect.left() > sidebar);

        if right {
            return;
        }
    }

    panic!("no control right of the sidebar took the focus in 60 Tabs");
}

fn every_page(theme: Theme, size: (f32, f32), name: &str) {
    let mut results = SnapshotResults::new();
    for id in pages() {
        let mut harness = page(id, theme, size, Setup::default());
        results.add(harness.try_snapshot(format!("coverage/{}_{name}", id.slug())));
    }
}

#[test]
fn every_page_in_light() {
    every_page(Theme::Light, WIDE, "light");
}

#[test]
fn every_page_at_the_smallest_window() {
    every_page(Theme::Dark, SMALLEST, "narrow");
}

#[test]
fn the_header_over_scrolled_content() {
    let mut results = SnapshotResults::new();
    for (theme, name) in [(Theme::Dark, "dark"), (Theme::Light, "light")] {
        let mut harness = page(PageId::Components, theme, WIDE, Setup::default());
        harness
            .get_by_role_and_label(egui::accesskit::Role::Button, "Install EasyEffects")
            .scroll_to_me();
        harness.run();
        results.add(harness.try_snapshot(format!("coverage/header_scrolled_{name}")));
    }
}

#[test]
fn every_page_with_the_focus_on_its_first_control() {
    let mut results = SnapshotResults::new();
    for id in pages() {
        let mut harness = page(id, Theme::Dark, WIDE, Setup::default());
        focus_page(&mut harness);
        results.add(harness.try_snapshot(format!("coverage/{}_focus", id.slug())));
    }
}
