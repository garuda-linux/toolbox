use egui::Rect;
use egui_kittest::Harness;

use super::super::{ActionRow, ButtonKind, Tone, card, card_row, pill_button};
use crate::fonts::FontFile;
use crate::icons;
use crate::testing::{filled_rect, harness_with_font, painted_text, test_fonts};
use crate::tokens::{self, set_page_hue};

/// How far the top of a title's letters may be from the tile's.
const LETTERS: f32 = 1.5;

const TITLE: &str = "Update the system";

/// The first pixel row of light letters inside `title`, read from the
/// rendered picture: light text on the dark card.
fn letters_top(harness: &mut Harness<'static, Rect>, title: Rect) -> f32 {
    let image = harness.render().expect("the wgpu test renderer draws");
    let scale = harness.ctx.pixels_per_point();
    let columns = pixel(title.left() * scale)..pixel(title.right() * scale);
    let mut rows = pixel(title.top() * scale)..pixel(title.bottom() * scale);

    let light = |x: u32, y: u32| {
        let [red, green, blue, _] = image.get_pixel(x, y).0;
        u16::from(red) + u16::from(green) + u16::from(blue) > 3 * 170
    };

    let first = rows
        .find(|y| columns.clone().any(|x| light(x, *y)))
        .unwrap_or_else(|| panic!("no letters in {title:?}"));

    f32::from(u16::try_from(first).unwrap_or(u16::MAX)) / scale
}

/// A position in the small test window as a whole pixel.
#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the test window is a few hundred whole pixels wide"
)]
const fn pixel(position: f32) -> u32 {
    position.round() as u32
}

/// Where an action row `width` wide put its tile and the top of its
/// title's letters, for a detail of `lines` lines, and the row's content.
fn placed(width: f32, lines: u8) -> (Rect, f32, Rect) {
    placed_in(None, width, lines)
}

fn placed_in(font: Option<FontFile>, width: f32, lines: u8) -> (Rect, f32, Rect) {
    let detail = (0..lines)
        .map(|line| format!("Line {line}"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut harness = harness_with_font(width, 400.0, font).build_ui_state(
        move |ui, content: &mut Rect| {
            set_page_hue(ui.ctx(), tokens::Hue::Green);
            card(ui, |ui| {
                let drawn = card_row(ui, true, |ui| {
                    ActionRow::new(icons::CHECKMARK, Tone::Neutral, TITLE)
                        .detail(&detail)
                        .show(ui, |ui| {
                            _ = pill_button(ui, "Run now", ButtonKind::Soft);
                        });
                });

                let padding = tokens::current(ui.ctx()).sizes.row_padding.y;
                *content = drawn.response.rect.shrink2(egui::vec2(0.0, padding));
            });
        },
        Rect::NOTHING,
    );

    harness.run();
    let fill = tokens::current(&harness.ctx)
        .tiles
        .of(tokens::Hue::Green)
        .fill;

    let badge = filled_rect(&harness, fill).expect("the tile is painted");
    let shape = painted_text(&harness, TITLE).expect("the title is painted");
    let title = shape.galley.rect.translate(shape.pos.to_vec2());
    let letters = letters_top(&mut harness, title);

    (badge, letters, *harness.state())
}

#[test]
fn the_tile_starts_at_the_titles_letters_beside_or_above_the_actions() {
    for width in [800.0, 300.0] {
        for lines in 0..=3 {
            let (badge, letters, content) = placed(width, lines);

            assert!(
                (badge.top() - letters).abs() < LETTERS,
                "{width} wide, {lines} lines: the tile starts at {}, the letters at {letters}",
                badge.top()
            );
            assert!(
                badge.bottom() <= content.bottom() + 0.5,
                "{width} wide, {lines} lines: the tile ends at {}, the row's content at {}",
                badge.bottom(),
                content.bottom()
            );
        }
    }
}

/// Up to one line of detail the tile is the row's tallest part. Longer
/// words move it down with the capitals, which `row_centre` checks.
#[test]
fn the_tile_keeps_the_rows_padding_above_and_below_while_it_is_tallest() {
    for width in [800.0, 300.0] {
        for lines in 0..=1 {
            let (badge, _, content) = placed(width, lines);

            assert!(
                (badge.top() - content.top()).abs() < 0.5,
                "{width} wide, {lines} lines: the tile starts at {}, the row's content at {}",
                badge.top(),
                content.top()
            );
        }
    }

    // A row as tall as its tile ends with it, so the tile has the same
    // room to the card's edge below as above.
    let (badge, _, content) = placed(800.0, 0);
    assert!(
        (badge.bottom() - content.bottom()).abs() < 0.5,
        "the tile ends at {}, the row's content at {}",
        badge.bottom(),
        content.bottom()
    );
}

#[test]
fn the_tile_starts_at_the_titles_letters_in_every_desktop_font() {
    for font in test_fonts() {
        let name = font
            .as_ref()
            .map_or("the bundled fonts", |font| &font.name)
            .to_owned();

        for lines in 0..=2 {
            let (badge, letters, _) = placed_in(font.clone(), 800.0, lines);

            assert!(
                (badge.top() - letters).abs() < LETTERS,
                "{name}, {lines} lines: the tile starts at {}, the letters at {letters}",
                badge.top()
            );
        }
    }
}
