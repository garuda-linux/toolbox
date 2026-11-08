//! Row heights from the fonts in use, after spotifast's
//! `card_row_height`: a larger desktop font makes the rows taller instead
//! of cutting their text.

use egui::Ui;

use super::super::layout::{capitals_in_line, last_descent, slack_below};
use crate::fonts::{self, BODY, SMALL, Weight};
use crate::theme::PILL_HEIGHT;
use crate::tokens;

/// The height of one line of text in `weight` at `scale` times the body.
pub(in crate::widgets) fn text_line_height(ui: &Ui, weight: Weight, scale: f32) -> f32 {
    let font = fonts::font(ui.style(), weight, scale);
    ui.ctx().fonts_mut(|fonts| fonts.row_height(&font))
}

/// A title line, with room for the tags and controls beside it, and
/// `lines` lines of small text below, as the rows of a card stack them.
pub fn text_block_height(ui: &Ui, lines: u8) -> f32 {
    // A card row's text is at least as tall as a control, with or without
    // one beside it (`split_row`).
    let text = text_line_height(ui, Weight::SemiBold, BODY);
    let line = text.max(PILL_HEIGHT);

    if lines == 0 {
        return line;
    }

    // The title line is raised to the last line's descent above its
    // capitals, and the lines below start under its text, as
    // `title_line` leaves them.
    let gap = tokens::current(ui.ctx()).sizes.tight_gap;
    let font = fonts::font(ui.style(), Weight::SemiBold, BODY);
    let rise = capitals_in_line(ui, font, false, line) - last_descent(ui);
    let title = line - rise - slack_below(line, text);
    let small = text_line_height(ui, Weight::Regular, SMALL);
    small.mul_add(f32::from(lines), title + gap).max(line)
}

/// A row around `content` points of content: the row padding above and
/// below.
pub fn row_height(ui: &Ui, content: f32) -> f32 {
    let padding = tokens::current(ui.ctx()).sizes.row_padding;
    padding.y.mul_add(2.0, content)
}
