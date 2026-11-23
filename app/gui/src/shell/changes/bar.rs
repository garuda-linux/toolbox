//! The footer under the page: the count, Review and Apply on a rule, as the
//! footer of a KDE settings module.

use egui::{Area, Context, CornerRadius, Id, Order, Pos2, Rect, Stroke, Ui, UiBuilder};
use toolbox_core::ChangeView;
use toolbox_gui_kit::fonts::{BODY, Weight};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::rich;

use super::buttons::{apply_button, group_width, review_button, show_console_button};
use super::{ChangesState, Queued, count_words, overlays};
use crate::command::CommandSender;

/// Under the dialogs' layer, so a dialog's scrim covers the footer as it
/// covers the page. egui raises an area in the first frame it shows, which
/// on the dialogs' layer put a returning footer over a dialog's buttons.
const LAYER: Order = Order::Middle;

/// `KCMUtils` footer margin on every side of the buttons.
const MARGIN: f32 = 6.0;

/// In `band`, the footer under the page, while the Console is closed. Its
/// area is the band, so the page above it never lies under it.
pub fn footer(
    ctx: &Context,
    band: Rect,
    queued: Queued<'_>,
    state: &mut ChangesState,
    commands: &CommandSender,
) {
    if queued.changes.is_empty() {
        *state = ChangesState::default();
        return;
    }

    let id = Id::unique("toolbox_gui::changes::footer");
    let shown = Area::new(id)
        .order(LAYER)
        .fixed_pos(band.min)
        .movable(false)
        .show(ctx, |ui| bar(ui, band, queued, state, commands));

    let layer = shown.response.layer_id;
    ctx.move_to_top(layer);
    overlays(ctx, (shown.inner, layer), queued, state, commands);
}

/// The footer's row under a rule: the count at the left, Review and Apply
/// at the right, with `KCMUtils` margins. Review is the popover's anchor.
fn bar(
    ui: &mut Ui,
    band: Rect,
    queued: Queued<'_>,
    state: &mut ChangesState,
    commands: &CommandSender,
) -> Rect {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    ui.set_min_size(band.size());
    ui.painter()
        .rect_filled(band, CornerRadius::ZERO, tokens.band);
    ui.painter().hline(
        band.x_range(),
        band.top() + 0.5,
        Stroke::new(1.0, tokens.separator),
    );

    let inner = row_rect(band, f32::from(tokens.sizes.page_padding.left));

    let removes = queued.changes.iter().any(ChangeView::removes);
    let mut anchor = inner;

    // A child does not advance the footer's cursor, so the footer's area
    // keeps the band's size and no more.
    let mut row = ui.new_child(UiBuilder::new().max_rect(inner));
    row.horizontal_centered(|line| {
        line.label(count_text(line, queued));

        // Open, the list carries the one Apply.
        let apply_shown = !state.open;
        let room = line.available_width() - group_width(line, apply_shown);
        line.add_space(room.max(0.0));

        // Review comes before Apply, so the keyboard reaches them in the
        // order they show.
        let toggle = if queued.running && !queued.console_open {
            show_console_button(line, commands)
        } else {
            review_button(line, state)
        };

        anchor = toggle.rect;

        if apply_shown {
            let (_, show) = apply_button(line, queued.held, commands, removes);
            if show {
                state.open = true;
            }
        }
    });

    anchor
}

/// The row inside the band. Its sides are the page body's, so the count and
/// Apply sit on the same edges as the cards above.
fn row_rect(band: Rect, side: f32) -> Rect {
    Rect::from_min_max(
        Pos2::new(band.left() + side, band.top() + 1.0 + MARGIN),
        Pos2::new(band.right() - side, band.bottom() - MARGIN),
    )
}

#[cfg(test)]
mod tests;

/// "2 changes queued", as plain text, since a count is no state.
pub(super) fn count_text(ui: &Ui, queued: Queued<'_>) -> egui::RichText {
    let ctx = ui.ctx();
    let count = queued.changes.len();
    let text = format!("{count} {}", count_words(ctx, count, queued.running));
    rich(ui, &text, Weight::Regular, BODY, tokens::current(ctx).text)
}
