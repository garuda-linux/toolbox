//! The header band blurs the page under it live: what scrolls under the
//! band shows in it the same frame.

use egui::{Color32, Rect, Sense, Ui, pos2, vec2};

use super::page_layout;
use crate::testing::{blurred_rects, harness};

/// A page of tall blocks, red and blue in turn, under a one-line header.
fn striped_page() -> egui_kittest::Harness<'static> {
    let mut harness = harness(400.0, 300.0).build_ui(|root| {
        page_layout(
            root,
            "page",
            &mut (),
            |(), ui: &mut Ui| {
                ui.label("Title");
                true
            },
            |(), ui: &mut Ui| {
                for row in 0_u8..20 {
                    let colour = if row % 2 == 0 {
                        Color32::RED
                    } else {
                        Color32::BLUE
                    };

                    let (rect, _) = ui.allocate_exact_size(vec2(360.0, 120.0), Sense::hover());
                    ui.painter().rect_filled(rect, 0.0, colour);
                }
            },
        );
    });

    harness.run();
    harness
}

fn band(harness: &egui_kittest::Harness<'static>) -> Rect {
    blurred_rects(harness)
        .into_iter()
        .min_by(|one, other| one.top().total_cmp(&other.top()))
        .expect("a blurred header band at the top")
}

/// How much redder than blue the band shows, summed over its pixels.
fn redness(harness: &mut egui_kittest::Harness<'static>, band: Rect) -> i64 {
    let image = harness.render().expect("the wgpu test renderer draws");
    let point = |value: u32| u16::try_from(value).map_or(f32::MAX, f32::from);

    image
        .enumerate_pixels()
        .filter(|(x, y, _)| band.contains(pos2(point(*x) + 0.5, point(*y) + 0.5)))
        .map(|(_, _, pixel)| {
            let [red, _, blue, _] = pixel.0;
            i64::from(red) - i64::from(blue)
        })
        .sum()
}

fn scroll(harness: &mut egui_kittest::Harness<'static>, points: f32) {
    harness.event(egui::Event::PointerMoved(pos2(200.0, 200.0)));
    harness.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: vec2(0.0, -points),
        modifiers: egui::Modifiers::NONE,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
    });
    harness.run();
}

#[test]
fn the_header_band_is_blurred() {
    let harness = striped_page();
    assert_ne!(blurred_rects(&harness).len(), 0);
}

#[test]
fn the_band_follows_what_scrolls_under_it() {
    let mut harness = striped_page();
    scroll(&mut harness, 60.0);
    let band = band(&harness);
    let before = redness(&mut harness, band);

    scroll(&mut harness, 120.0);
    let after = redness(&mut harness, band);

    assert!(
        (before > 0) != (after > 0),
        "the band did not change colour: {before} then {after}"
    );
}
