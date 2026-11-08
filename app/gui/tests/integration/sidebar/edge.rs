use egui::{Pos2, Theme};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui::page::PageId;
use toolbox_gui::{App, Setup};
use toolbox_gui_kit::testing::lines_in;
use toolbox_gui_kit::theme::SIDEBAR_WIDTH;
use toolbox_gui_kit::tokens;

use crate::common::{HEIGHT, WIDTH, launch_in};

/// The app on the Users page, which has a header band.
fn app(theme: Theme, pixels_per_point: f32) -> Harness<'static, App> {
    let builder = toolbox_gui_kit::testing::harness(WIDTH, HEIGHT)
        .with_theme(theme)
        .with_pixels_per_point(pixels_per_point);

    let setup = Setup {
        open: Some(PageId::Users),
        ..Setup::default()
    };

    launch_in(builder, setup)
}

fn dividers(harness: &Harness<'_, App>) -> Vec<[Pos2; 2]> {
    let outline = tokens::current(&harness.ctx).outline;
    lines_in(harness, outline)
        .into_iter()
        .filter(|[one, other]| (one.x - other.x).abs() < 0.01)
        .filter(|[one, other]| one.y.min(other.y) <= 0.5 && one.y.max(other.y) >= HEIGHT - 0.5)
        .collect()
}

fn parted_at(harness: &Harness<'_, App>, edge: f32) -> bool {
    let pixels = harness.ctx.pixels_per_point();
    let column = (edge * pixels).round() - 0.5;
    dividers(harness)
        .iter()
        .any(|[top, _]| top.x.mul_add(pixels, -column).abs() < 0.01)
}

#[test]
fn a_hairline_parts_the_sidebar_from_the_page_in_either_theme() {
    for theme in [Theme::Dark, Theme::Light] {
        let harness = app(theme, 1.0);
        assert!(
            parted_at(&harness, SIDEBAR_WIDTH),
            "{theme:?}: no divider at {SIDEBAR_WIDTH}, only {:?}",
            dividers(&harness)
        );
    }
}

#[test]
fn the_hairline_stays_on_one_pixel_column_at_a_fractional_scale() {
    let harness = app(Theme::Dark, 1.5);
    assert!(
        parted_at(&harness, SIDEBAR_WIDTH),
        "no crisp divider, only {:?}",
        dividers(&harness)
    );
}

#[test]
fn the_collapsed_rail_is_parted_from_the_page_too() {
    let mut harness = app(Theme::Light, 1.0);
    harness.get_by_label("Collapse sidebar").click();
    harness.run();
    harness.run();

    let rail = toolbox_gui::testing::COLLAPSED_SIDEBAR;
    assert!(
        parted_at(&harness, rail),
        "no divider at {rail}, only {:?}",
        dividers(&harness)
    );
}

#[test]
fn the_header_band_starts_right_beside_the_hairline() {
    let harness = app(Theme::Dark, 1.0);
    let scrim = tokens::current(&harness.ctx).header_scrim;
    let band = harness
        .output()
        .shapes
        .iter()
        .find_map(|clipped| {
            if let egui::Shape::Rect(rect) = &clipped.shape
                && rect.fill == scrim
                && rect.rect.top() <= 0.5
            {
                Some(rect.rect)
            } else {
                None
            }
        })
        .expect("the header band is painted");

    assert!(
        (band.left() - SIDEBAR_WIDTH).abs() < 0.01,
        "the band {band:?} does not meet the divider's side at {SIDEBAR_WIDTH}"
    );
}
