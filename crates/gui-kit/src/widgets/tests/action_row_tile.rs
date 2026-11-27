use egui::Rect;

use super::super::{ActionRow, ButtonKind, card, card_row, pill_button};
use crate::fonts::FontFile;
use crate::icons;
use crate::testing::{harness_with_font, painted_text, test_fonts, textured_rects};
use crate::tokens::{self, set_page_hue};

/// Two centres count as one, within the ink of a line rather than its box.
const TEXT: f32 = 1.0;

const SAME: f32 = 0.5;

/// A glyph snaps to whole pixels, so its centre can be half a point off.
const SNAP: f32 = 1.0;

/// Letters without a descender, so the title's ink bottom is its baseline.
const TITLE: &str = "Lutris";

/// An action row with `detail` in a card, and the row's content, inside
/// its padding. Shared with `row_centre`.
pub(super) fn row_content(ui: &mut egui::Ui, detail: &str) -> Rect {
    set_page_hue(ui.ctx(), tokens::Hue::Green);
    let mut content = Rect::NOTHING;
    card(ui, |ui| {
        let drawn = card_row(ui, true, |ui| {
            ActionRow::new(icons::CHECKMARK, TITLE)
                .detail(detail)
                .show(ui, |ui| {
                    _ = pill_button(ui, "Run now", ButtonKind::Soft);
                });
        });

        let padding = tokens::current(ui.ctx()).sizes.row_padding.y;
        content = drawn.response.rect.shrink2(egui::vec2(0.0, padding));
    });

    content
}

/// Where an action row `width` wide drew its icon, its text block (the
/// title's capitals to the last baseline) and its row's content, for a
/// detail of `lines` lines.
fn placed(width: f32, lines: u8) -> (Rect, (f32, f32), Rect) {
    placed_in(None, width, lines)
}

/// The top and the bottom of a line's ink, from its galley's glyph bounds.
pub(super) fn ink(painted: &egui::epaint::TextShape) -> (f32, f32) {
    let bounds = painted.galley.mesh_bounds;
    (
        painted.pos.y + bounds.top(),
        painted.pos.y + bounds.bottom(),
    )
}

fn placed_in(font: Option<FontFile>, width: f32, lines: u8) -> (Rect, (f32, f32), Rect) {
    let detail = (0..lines)
        .map(|line| format!("Line {line}"))
        .collect::<Vec<_>>()
        .join("\n");

    let shown = detail.clone();
    let mut harness = harness_with_font(width, 400.0, font).build_ui_state(
        move |ui, content: &mut Rect| *content = row_content(ui, &shown),
        Rect::NOTHING,
    );

    harness.run();
    let last = if lines == 0 { TITLE } else { detail.as_str() };

    let badge = textured_rects(&harness)
        .first()
        .copied()
        .expect("the icon is painted");

    let title = painted_text(&harness, TITLE).expect("the title is painted");
    let last = painted_text(&harness, last).expect("the last line is painted");
    let text = (ink(&title).0, ink(&last).1);

    (badge, text, *harness.state())
}

#[test]
fn the_icon_and_the_text_block_centre_on_the_row_beside_the_actions() {
    for lines in 0..=3 {
        let (badge, (top, bottom), content) = placed(800.0, lines);

        assert!(
            (badge.center().y - content.center().y).abs() < SNAP,
            "800 wide, {lines} lines: the icon centre is {}, the row's at {}",
            badge.center().y,
            content.center().y
        );
        assert!(
            (top.midpoint(bottom) - content.center().y).abs() < TEXT,
            "800 wide, {lines} lines: the text block is {top} to {bottom}, the row's centre at {}",
            content.center().y
        );
    }
}

/// Stacked under the words the actions leave the centring out, so the icon
/// centres on the text block alone.
#[test]
fn the_icon_centres_on_the_text_block_when_the_actions_go_under_it() {
    for lines in 0..=3 {
        let (badge, (top, bottom), content) = placed(300.0, lines);

        assert!(
            (badge.center().y - top.midpoint(bottom)).abs() < TEXT,
            "300 wide, {lines} lines: the icon centre is {}, the text block's at {}",
            badge.center().y,
            top.midpoint(bottom)
        );
        assert!(
            badge.top() >= content.top() - SAME && badge.bottom() <= content.bottom() + SAME,
            "300 wide, {lines} lines: the icon {badge:?} leaves the row's content {content:?}"
        );
    }
}

#[test]
fn the_icon_centres_on_the_text_block_in_every_desktop_font() {
    for font in test_fonts() {
        let name = font
            .as_ref()
            .map_or("the bundled fonts", |font| &font.name)
            .to_owned();

        for lines in 0..=2 {
            let (badge, (top, bottom), content) = placed_in(font.clone(), 800.0, lines);

            assert!(
                (badge.center().y - content.center().y).abs() < SNAP,
                "{name}, {lines} lines: the icon centre is {}, the row's at {}",
                badge.center().y,
                content.center().y
            );
            assert!(
                (top.midpoint(bottom) - content.center().y).abs() < TEXT,
                "{name}, {lines} lines: the text block is {top} to {bottom}, the row's centre at {}",
                content.center().y
            );
        }
    }
}
