//! A row's title line.
//!
//! In a card's row the line is as tall as a control, so every row keeps
//! one middle line with or without one. A tag on the line sits on its
//! middle, its words on the baseline of a status dot's words.
//!
//! At the top of a card's row the line is raised until the title's
//! capitals start the last line's descent below the row's top, and it
//! takes room only down to its text. The room above the capitals is then
//! the room under the last baseline, where the room of the line and the
//! font's own room above the capitals would add to it.

use egui::{Align, Id, Layout, Ui, UiBuilder, vec2};

use super::super::card::in_card_row;
use super::{SETTLED, line_height};
use crate::fonts::{self, BODY, SMALL, Weight};

/// At the top of a card's row the title's capitals start as far under
/// the row's top as the last line's baseline ends above its bottom.
pub fn title_line<R>(ui: &mut Ui, wrap: bool, add: impl FnOnce(&mut Ui) -> R) -> R {
    let text = ui.text_style_height(&egui::TextStyle::Body);
    let line = title_line_height(ui);
    let lines = |line_ui: &mut Ui| {
        line_ui.spacing_mut().interact_size.y = line;
        if wrap {
            line_ui.horizontal_wrapped(add).inner
        } else {
            line_ui.horizontal(add).inner
        }
    };

    let slack = slack_below(line, text);
    if !(in_card_row(ui) && ui.layout().main_dir() == egui::Direction::TopDown) {
        return ui.scope(lines).inner;
    }

    // Below other lines the title keeps its line, as raised it would lie
    // over them.
    if ui.cursor().top() - ui.max_rect().top() >= SETTLED {
        let drawn = ui.scope(lines);
        ui.add_space(-last_row_slack(drawn.response.rect, line, slack));
        return drawn.inner;
    }

    let rise = capitals_in_line(ui, title_font(ui), wrap, line) - last_descent(ui);
    let room = ui.available_rect_before_wrap();
    let column = egui::Rect::from_min_size(
        room.min - vec2(0.0, rise),
        vec2(room.width(), room.height() + rise),
    );

    let mut child = ui.new_child(
        UiBuilder::new()
            .id_salt(ui.next_auto_id().with("title_line"))
            .max_rect(column)
            .layout(Layout::top_down(Align::Min)),
    );

    let inner = lines(&mut child);
    let taken = child.min_rect();
    let single = one_row(taken, line);
    let slack = last_row_slack(taken, line, slack);
    let below = (taken.bottom() - slack - room.top()).max(0.0);
    let (end, _) = ui.allocate_exact_size(vec2(taken.width(), below), egui::Sense::hover());

    // The raised line starts at its capitals. Down to as far below the
    // line's middle as they start above it, it is centred as before. A
    // wrapped title has no tail, as its rows below the first are text.
    let tail = if single {
        below - 2.0_f32.mul_add(-rise, line)
    } else {
        0.0
    };

    ui.data_mut(|data| data.insert_temp(tail_id(), [end.bottom(), tail]));
    inner
}

/// egui lays a wrapped title's first row as tall as the line and every
/// later row as tall as its text, so only a title on one row ends in the
/// line's slack.
fn one_row(taken: egui::Rect, line: f32) -> bool {
    taken.height() <= line + SETTLED
}

fn last_row_slack(taken: egui::Rect, line: f32, slack: f32) -> f32 {
    if one_row(taken, line) { slack } else { 0.0 }
}

pub fn title_line_height(ui: &Ui) -> f32 {
    let text = ui.text_style_height(&egui::TextStyle::Body);
    line_height(ui, text).max(text)
}

fn title_font(ui: &Ui) -> egui::FontId {
    fonts::font(ui.style(), Weight::SemiBold, BODY)
}

/// A picture beside the title starts here to meet its letters.
pub fn title_capitals_top(ui: &Ui, font: egui::FontId, wrap: bool) -> f32 {
    let line = title_line_height(ui);
    let raised = if in_card_row(ui) {
        capitals_in_line(ui, title_font(ui), wrap, line) - last_descent(ui)
    } else {
        0.0
    };

    capitals_in_line(ui, font, wrap, line) - raised
}

/// A single line centres the text in its height. A wrapping one lays its
/// first row as tall as the line, so an "H" is laid out the same way and
/// measured.
pub(in crate::widgets) fn capitals_in_line(
    ui: &Ui,
    font: egui::FontId,
    wrap: bool,
    line: f32,
) -> f32 {
    if wrap {
        let format = egui::TextFormat {
            valign: Align::Center,
            ..egui::TextFormat::simple(font, egui::Color32::PLACEHOLDER)
        };

        let mut job = egui::text::LayoutJob::single_section("H".to_owned(), format);
        job.first_row_min_height = line;

        return ui
            .fonts_mut(|fonts| fonts.layout_job(job))
            .mesh_bounds
            .top();
    }

    let capital = ui
        .painter()
        .layout_no_wrap("H".to_owned(), font, egui::Color32::PLACEHOLDER);

    (line - capital.rect.height()) / 2.0 + capital.mesh_bounds.top()
}

pub(super) fn tail_id() -> Id {
    Id::unique("toolbox_gui_kit::title_tail")
}

/// A title alone is centred on its capitals, as a control's text is, so
/// the room the font keeps under them must not count.
pub(super) fn title_tail(ui: &Ui, bottom: f32) -> f32 {
    ui.data(|data| data.get_temp::<[f32; 2]>(tail_id()))
        .filter(|[end, _]| (end - bottom).abs() < SETTLED)
        .map_or(0.0, |[_, tail]| tail)
}

/// How far what follows a title line `line` tall moves up into the line's
/// lower part, so it starts right under the title's `text`. A tag ends
/// with the text, so nothing beside the title reaches lower.
pub(in crate::widgets) fn slack_below(line: f32, text: f32) -> f32 {
    ((line - text) / 2.0).max(0.0)
}

/// How far a row's last line, in the small text, reaches below its
/// baseline. A card row keeps its capitals this much further from its
/// top, so the room above them is the room under that baseline.
pub(in crate::widgets) fn last_descent(ui: &Ui) -> f32 {
    let font = fonts::font(ui.style(), Weight::Regular, SMALL);
    let capital = ui
        .painter()
        .layout_no_wrap("H".to_owned(), font, egui::Color32::PLACEHOLDER);

    (capital.rect.height() - capital.mesh_bounds.bottom()).max(0.0)
}
