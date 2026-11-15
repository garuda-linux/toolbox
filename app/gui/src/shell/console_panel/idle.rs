//! The lines painted over the terminal: what an empty one is for, and how
//! to leave a focused one.

use egui::{Color32, CornerRadius, Rect, Sense, TextStyle, Ui, WidgetText, pos2, vec2};
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::theme::CARD_RADIUS;
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::rich;

use crate::i18n;

/// What an empty terminal is for, or after a run that it printed nothing,
/// two cells into its first line. It is painted over the terminal and
/// takes no room, so the sheet's height and slide stay the terminal's own.
///
/// `terminal` is the cell grid, inside the terminal's padding.
pub(super) fn idle_hint(ui: &Ui, terminal: Rect, ran: bool) {
    let tokens = tokens::current(ui.ctx());
    let id = if ran {
        "console-printed-nothing"
    } else {
        "console-idle-hint"
    };

    let text = i18n::text(ui.ctx(), id);
    let words = rich(ui, &text, Weight::Regular, BODY, tokens.secondary);

    // The same cell the terminal view sizes its grid by, so the hint sits on
    // the grid's columns, two cells in from where output starts.
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
    let rect = Rect::from_min_size(at, galley.size());
    let response = ui.interact(rect, ui.scope_id().with("idle-hint"), Sense::hover());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, &*text));
    ui.painter().galley(at, galley, tokens.secondary);
}

/// How to leave the focused terminal, on a tab in the terminal `frame`'s
/// `fill` that sits across its bottom edge at the right. It takes no room
/// below the frame, so the terminal reaches down to the sheet's padding
/// and focus never resizes it and with it the program.
pub(super) fn leave_hint(ui: &Ui, frame: Rect, hint: &str, fill: Color32) {
    let tokens = tokens::current(ui.ctx());
    let words = rich(ui, hint, Weight::Regular, SMALL, tokens.secondary);
    let pad = vec2(tokens.sizes.inset, tokens.sizes.tight_gap / 2.0);
    let inner = f32::from(tokens.sizes.terminal_padding.right);
    let width = (inner + pad.x).mul_add(-2.0, frame.width()).max(0.0);
    let galley = WidgetText::from(words).into_galley(
        ui,
        Some(egui::TextWrapMode::Truncate),
        width,
        TextStyle::Body,
    );

    let size = galley.size();
    let at = pos2(
        frame.right() - inner - size.x,
        frame.bottom() - size.y / 2.0,
    );

    let tab = Rect::from_min_size(at, size).expand2(pad);
    ui.painter()
        .rect_filled(tab, CornerRadius::same(CARD_RADIUS), fill);
    ui.painter().galley(at, galley, tokens.secondary);
}
