//! The one focus ring every painted widget shares, so keyboard users see
//! where they are on every control alike. Like a browser's
//! `:focus-visible`, a click focuses without the ring, and the next key
//! brings it back.

#[cfg(test)]
mod tests;

use std::sync::Arc;

use egui::{Context, CornerRadius, Event, Id, Rect, Response, Stroke, StrokeKind, Ui};

use crate::tokens;

const GAP: u8 = 2;

pub fn focus_ring(ui: &Ui, response: &Response, radius: u8) {
    if response.has_focus() {
        ring_around(ui, response.rect, radius);
    }
}

/// The ring around `rect`, for a control whose focus sits on a part of
/// it, such as a search field's text.
pub(super) fn ring_around(ui: &Ui, rect: Rect, radius: u8) {
    if !visible(ui) {
        return;
    }

    let accent = tokens::current(ui.ctx()).accent_ring;
    ui.painter().rect_stroke(
        rect.expand(f32::from(GAP)),
        CornerRadius::same(radius.saturating_add(GAP)),
        Stroke::new(2.0, accent),
        StrokeKind::Outside,
    );
}

/// Follows the pointer and the keys at the start of every pass.
///
/// A field takes the focus on the press and draws its ring the frames
/// after, while the button is still held, so a check only where a ring
/// draws would miss the click.
pub fn install(ctx: &Context) {
    let installed = Id::unique("toolbox_gui_kit::focus::installed");
    if ctx.data(|data| data.get_temp::<bool>(installed)).is_some() {
        return;
    }

    ctx.data_mut(|data| data.insert_temp(installed, true));
    ctx.on_begin_pass("toolbox-focus-visible", Arc::new(|ui| follow(ui.ctx())));
}

fn follow(ctx: &Context) {
    let (key, pointer) = ctx.input(|input| {
        let key = input.events.iter().any(|event| {
            matches!(
                event,
                Event::Key {
                    pressed: true,
                    key: _,
                    physical_key: _,
                    repeat: _,
                    modifiers: _,
                }
            )
        });

        (
            key,
            input.pointer.any_pressed() || input.pointer.any_released(),
        )
    });

    let shown = match (key, pointer) {
        (true, _) => Some(true),
        (false, true) => Some(false),
        (false, false) => None,
    };

    if let Some(shown) = shown {
        ctx.data_mut(|data| data.insert_temp(visible_id(), shown));
    }
}

/// Whether the focus came from the keyboard: a key since the last
/// click, or no click yet.
fn visible(ui: &Ui) -> bool {
    ui.data(|data| data.get_temp::<bool>(visible_id()))
        .unwrap_or(true)
}

fn visible_id() -> Id {
    Id::unique("toolbox_gui_kit::focus::visible")
}
