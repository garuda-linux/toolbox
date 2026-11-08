use egui::Ui;
use egui_kittest::kittest::Queryable as _;

use super::page_layout;
use crate::testing::{fill_under, filled_rects, harness, lines_in, textured_rects};
use crate::tokens;

fn long_page() -> egui_kittest::Harness<'static, u32> {
    let mut harness = harness(400.0, 300.0).build_ui_state(
        |root, clicks: &mut u32| {
            page_layout(
                root,
                "page",
                clicks,
                |_, ui: &mut Ui| {
                    ui.label("Title");
                    true
                },
                |clicks, ui: &mut Ui| {
                    for row in 0_u8..40 {
                        let item = ui.button(format!("Item {row}"));

                        // As the list rows do, so the keyboard keeps it in sight.
                        if item.gained_focus() {
                            item.scroll_to_me(None);
                        }

                        if item.clicked() {
                            *clicks += 1;
                        }
                    }
                },
            );
        },
        0,
    );

    harness.run();
    harness
}

fn scroll(harness: &mut egui_kittest::Harness<'static, u32>, points: f32) {
    harness.event(egui::Event::PointerMoved(egui::pos2(200.0, 200.0)));
    harness.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, -points),
        modifiers: egui::Modifiers::NONE,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
    });
    harness.run();
}

fn header_band(harness: &egui_kittest::Harness<'static, u32>) -> egui::Rect {
    textured_rects(harness)
        .into_iter()
        .min_by(|one, other| one.top().total_cmp(&other.top()))
        .expect("a blurred header band at the top")
}

fn header_bottom(harness: &egui_kittest::Harness<'static, u32>) -> f32 {
    let header = header_band(harness);

    assert!(header.top() < 10.0, "the band starts at {}", header.top());
    header.bottom()
}

#[test]
fn the_header_is_a_blurred_band_down_to_below_its_title() {
    let harness = long_page();
    let bottom = header_bottom(&harness);
    let title = harness.get_by_label("Title").rect();

    assert!(bottom > title.bottom(), "{bottom} against {title:?}");
    assert!(
        harness.get_by_label("Item 0").rect().top() >= bottom,
        "the first item starts under the band"
    );
}

#[test]
fn a_neutral_scrim_covers_the_blur_of_the_band() {
    let mut harness = long_page();
    scroll(&mut harness, 120.0);

    let blurred = header_band(&harness);

    let scrim = tokens::current(&harness.ctx).header_scrim;
    let title = harness.get_by_label("Title").rect();

    assert_eq!(
        filled_rects(&harness, scrim),
        [blurred],
        "the scrim lies over the whole blurred band"
    );
    assert_eq!(
        fill_under(
            &harness,
            egui::pos2(blurred.right() - 4.0, title.center().y)
        ),
        Some(scrim),
        "away from the text only the thin scrim lies"
    );
}

#[test]
fn the_header_text_lies_on_the_same_thin_scrim_as_the_band() {
    let mut harness = long_page();
    scroll(&mut harness, 120.0);

    let scrim = tokens::current(&harness.ctx).header_scrim;
    let title = harness.get_by_label("Title").rect();

    assert_eq!(fill_under(&harness, title.center()), Some(scrim));
}

#[test]
fn the_page_runs_under_the_header_once_scrolled() {
    let mut harness = long_page();
    scroll(&mut harness, 120.0);

    let bottom = header_bottom(&harness);
    let clip_top = harness
        .output()
        .shapes
        .iter()
        .filter(|clipped| matches!(clipped.shape, egui::Shape::Text(_)))
        .map(|clipped| clipped.clip_rect.top())
        .fold(f32::INFINITY, f32::min);

    assert!(
        clip_top < bottom,
        "the page stops at the header: {clip_top} against {bottom}"
    );
}

#[test]
fn a_hairline_marks_the_header_only_once_the_page_runs_under_it() {
    let mut harness = long_page();
    let hairline = tokens::current(&harness.ctx).outline;
    let at_rest = header_bottom(&harness);
    let marked = |shown: &egui_kittest::Harness<'static, u32>, bottom: f32| {
        lines_in(shown, hairline)
            .iter()
            .any(|[start, end]| (start.y - bottom).abs() < 1.5 && end.x - start.x > 200.0)
    };

    assert!(
        !marked(&harness, at_rest),
        "a hairline before anything runs under"
    );

    scroll(&mut harness, 120.0);
    let bottom = header_bottom(&harness);
    assert!(
        marked(&harness, bottom),
        "no hairline once the page runs under"
    );
}

#[test]
fn a_focused_control_comes_fully_below_the_header() {
    let mut harness = long_page();
    scroll(&mut harness, 600.0);

    harness.get_by_label("Item 2").focus();
    harness.run();

    let bottom = header_bottom(&harness);
    let item = harness.get_by_label("Item 2").rect();
    assert!(
        item.top() >= bottom,
        "{item:?} hides under the header ending at {bottom}"
    );
}

#[test]
fn a_click_on_the_header_does_not_reach_the_page_under_it() {
    let mut harness = long_page();
    scroll(&mut harness, 120.0);

    let bottom = header_bottom(&harness);
    let under = (0_u8..40)
        .map(|row| harness.get_by_label(&format!("Item {row}")).rect())
        .find(|rect| rect.center().y < bottom && rect.center().y > 0.0)
        .expect("an item under the header");

    harness.event(egui::Event::PointerMoved(under.center()));
    harness.run();
    harness.get_by_label("Title");
    harness.event(egui::Event::PointerButton {
        pos: under.center(),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    harness.event(egui::Event::PointerButton {
        pos: under.center(),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    harness.run();

    assert_eq!(*harness.state(), 0, "the click went through the header");
}

#[test]
fn the_body_starts_one_header_gap_below_the_band() {
    let harness = long_page();
    let bottom = header_bottom(&harness);
    let first = harness.get_by_label("Item 0").rect();
    let gap = tokens::current(&harness.ctx).sizes.header_gap;

    assert!(
        (first.top() - bottom - gap).abs() < 1.0,
        "the body starts {} under the band, not {gap}",
        first.top() - bottom
    );
}
