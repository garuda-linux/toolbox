//! The line an empty terminal shows in place of output.

use egui::{Sense, TextStyle, Ui, WidgetText, pos2, vec2};
use toolbox_gui_kit::fonts::{BODY, Weight};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::rich;

use crate::i18n;

/// What an empty terminal is for, or after a run that it printed nothing,
/// past the cursor on its first line. It is painted over the terminal and
/// takes no room, so the sheet's height and slide stay the terminal's own.
///
/// `terminal` is the cell grid, the rect the terminal view drew its cursor
/// in, inside the terminal's padding.
pub(super) fn idle_hint(ui: &Ui, terminal: egui::Rect, ran: bool) {
    let tokens = tokens::current(ui.ctx());
    let id = if ran {
        "console-printed-nothing"
    } else {
        "console-idle-hint"
    };

    let text = i18n::text(ui.ctx(), id);
    let words = rich(ui, &text, Weight::Regular, BODY, tokens.secondary);

    // The same cell the terminal view sizes its grid by, so the hint keeps
    // one empty cell between itself and the cursor.
    let font = TextStyle::Monospace.resolve(ui.style());
    let cell = ui.fonts_mut(|fonts| vec2(fonts.glyph_width(&font, 'M'), fonts.row_height(&font)));
    let left = cell.x.mul_add(2.0, terminal.left());

    let width = (terminal.right() - left).max(0.0);
    let galley = WidgetText::from(words).into_galley(
        ui,
        Some(egui::TextWrapMode::Truncate),
        width,
        TextStyle::Body,
    );

    let at = pos2(left, terminal.top() + (cell.y - galley.size().y) / 2.0);
    let rect = egui::Rect::from_min_size(at, galley.size());
    let response = ui.interact(rect, ui.scope_id().with("idle-hint"), Sense::hover());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, &*text));
    ui.painter().galley(at, galley, tokens.secondary);
}
