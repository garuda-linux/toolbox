//! The page under the open Console: it paints on under the sheet, so the
//! sheet's blur shows it, scrolls every row up above the sheet, and
//! takes no click that lands on the sheet.

use egui::accesskit::Role;
use egui::epaint::ClippedShape;
use egui::{Event, Modifiers, PointerButton, Pos2, Rect, Shape};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_gui::App;

use super::{app, grip};
use crate::common::{drag, open_console, pane, wheel};
use crate::maintenance::maintenance;

/// The left edge of the page beside the sidebar, in points.
const PAGE_LEFT: f32 = 260.0;

/// The shapes the page painted under the sheet: their clip starts at the
/// page's top, as the sheet's own starts at the sheet's.
fn page_under(harness: &Harness<'static, App>, sheet: Rect) -> Vec<ClippedShape> {
    let page = pane(harness, "Page").expect("the page");
    harness
        .output()
        .shapes
        .iter()
        .filter(|clipped| {
            let bounds = clipped.shape.visual_bounding_rect();
            clipped.clip_rect.top() <= page.top() + 1.0
                && clipped.clip_rect.bottom() > sheet.top() + 1.0
                && bounds.left() > PAGE_LEFT
                && bounds.top() > sheet.top()
                && bounds.intersects(sheet)
        })
        .cloned()
        .collect()
}

/// How many of the page's texts lie under the sheet.
fn texts_under(harness: &Harness<'static, App>, sheet: Rect) -> usize {
    page_under(harness, sheet)
        .iter()
        .filter(|clipped| matches!(clipped.shape, Shape::Text(_)))
        .count()
}

/// The page's links, the Home tiles, with where they lie.
fn page_links(harness: &Harness<'static, App>) -> Vec<(String, Rect)> {
    harness
        .get_all_by_role(Role::Link)
        .filter(|node| node.rect().left() > PAGE_LEFT)
        .map(|node| {
            (
                node.accesskit_node().label().unwrap_or_default(),
                node.rect(),
            )
        })
        .collect()
}

#[test]
fn the_page_paints_on_under_the_sheet() {
    let mut fixture = maintenance();
    let harness = fixture.harness();
    let sheet = open_console(harness);

    assert!(
        texts_under(harness, sheet) > 0,
        "no page row painted under the sheet at {sheet:?}"
    );
}

#[test]
fn the_sheets_blur_gathers_the_page_under_it() {
    let mut fixture = maintenance();
    let harness = fixture.harness();
    let sheet = open_console(harness);

    // The blur leaves text out, so it needs the page's other shapes, the
    // rows' icons and fills, under the sheet.
    let gathered = page_under(harness, sheet)
        .iter()
        .filter(|clipped| !matches!(clipped.shape, Shape::Text(_) | Shape::Callback(_)))
        .count();

    assert!(gathered > 0, "nothing of the page under the sheet to blur");
}

#[test]
fn the_page_scrolls_its_last_row_above_the_sheet() {
    let mut fixture = maintenance();
    let harness = fixture.harness();
    let sheet = open_console(harness);

    for _ in 0_u8..6 {
        wheel(
            harness,
            Pos2::new(600.0, sheet.top() - 60.0),
            egui::vec2(0.0, -2000.0),
        );
        harness.run();
    }

    assert_eq!(
        texts_under(harness, sheet),
        0,
        "page rows stay under the sheet from {} at the end",
        sheet.top()
    );
}

#[test]
fn a_click_on_the_sheet_never_reaches_the_page_under_it() {
    let mut harness = app();
    open_console(&mut harness);

    // Dragged as tall as it gets, the sheet covers the Home tiles.
    let from = grip(&harness);
    drag(&mut harness, from, Pos2::new(from.x, 0.0));
    harness.run();
    let sheet = pane(&harness, "Console").expect("open");

    let (label, tile) = page_links(&harness)
        .into_iter()
        .find(|(_, rect)| rect.top() > sheet.top() + 30.0)
        .expect("a tile under the sheet");

    let at = tile.center();
    let button = |pressed| Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };

    harness.event(Event::PointerMoved(at));
    harness.run();
    harness.event(button(true));
    harness.run();
    harness.event(button(false));
    harness.run();

    assert!(
        page_links(&harness)
            .iter()
            .any(|(shown, rect)| *shown == label && rect == &tile),
        "the click on the sheet opened {label}"
    );
}
