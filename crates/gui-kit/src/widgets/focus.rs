//! Keyboard focus for painted widgets. Darkly draws no focus ring, so a
//! focused control shows its focus as a fill or a line. Like a browser's
//! `:focus-visible`, it shows only where Tab, Shift+Tab or an arrow key
//! moved the focus. A click or the program focuses without showing it.

#[cfg(test)]
mod tests;

use std::sync::Arc;

use egui::epaint::RectShape;
use egui::{
    Context, CornerRadius, Event, Id, Key, Rect, Response, Shadow, Shape, Stroke, StrokeKind, Ui,
};

use super::buttons::look::BLUR_PER_RADIUS;
use crate::tokens::{self, Tokens};

/// Where the keyboard put the focus.
#[derive(Clone, Copy, Default)]
struct Keyboard {
    /// The widget a key moved the focus to, while it still holds it.
    holder: Option<Id>,
    /// A key that moves the focus came in this pass. Tab moves it while
    /// the pass runs and an arrow key once it ends, so the widget it
    /// lands on is known only in the next pass.
    moving: bool,
}

/// Whether `response` holds the focus and Tab, Shift+Tab or an arrow key
/// brought it there, as a button's fill shows it. A focus the program or
/// a click gave shows no fill, whatever keys came before it.
pub fn focus_from_keyboard(ui: &Ui, response: &Response) -> bool {
    response.has_focus() && from_keys(ui, response.id)
}

/// The ring on `rect`'s own edge while `response` has the focus from the
/// keyboard, for a box that fills its room to the edge, such as the
/// Console sheet's drag bar, where a ring outside would leave the room.
pub fn edge_ring(ui: &Ui, response: &Response, rect: Rect, radius: u8) {
    if !focus_from_keyboard(ui, response) {
        return;
    }

    let accent = tokens::current(ui.ctx()).accent_ring;
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(radius),
        Stroke::new(2.0, accent),
        StrokeKind::Inside,
    );
}

/// Darkly's halo of a focused field (darklyhelper.cpp:1064-1066).
pub(super) fn field_halo(tokens: &Tokens, fill: Rect, radius: u8) -> [Shape; 2] {
    let shadow = Shadow {
        offset: [0, 1],
        blur: 7 * BLUR_PER_RADIUS,
        spread: 0,
        color: tokens.control_shadow,
    };

    let ring = RectShape::filled(
        fill.expand(1.0),
        radius.saturating_add(1),
        tokens.accent_tint.gamma_multiply(0.6),
    );

    [
        Shape::from(shadow.as_shape(fill.shrink(1.0), radius)),
        Shape::from(ring),
    ]
}

/// Darkly's focus line of an unselected list item (darklystyle.cpp:4183-4222),
/// which Darkly skips under 10 px wide.
pub fn item_focus_line(ui: &Ui, response: &Response, rect: Rect, selected: bool) {
    if selected || rect.width() < 10.0 || !focus_from_keyboard(ui, response) {
        return;
    }

    let line = tokens::current(ui.ctx()).accent_tint;
    ui.painter()
        .hline(rect.x_range(), rect.bottom() - 0.5, Stroke::new(1.0, line));
}

/// Follows the pointer, the keys and egui's focus at the start of every
/// pass.
///
/// A field takes the focus on the press and draws its halo the frames
/// after, while the button is still held, so a check only where a halo
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
    let (moving, pointer) = ctx.input(|input| {
        (
            input.events.iter().any(moves_focus),
            input.pointer.any_pressed() || input.pointer.any_released(),
        )
    });

    let focused = ctx.memory(egui::Memory::focused);
    let before = ctx.data(|data| data.get_temp::<Keyboard>(keyboard_id()).unwrap_or_default());
    let holder = if pointer {
        None
    } else if before.moving {
        focused
    } else {
        before.holder.filter(|holder| Some(*holder) == focused)
    };

    let now = Keyboard {
        holder,
        moving: moving && !pointer,
    };

    ctx.data_mut(|data| data.insert_temp(keyboard_id(), now));
    if now.moving {
        ctx.request_repaint();
    }
}

/// Tab, Shift+Tab and the plain arrow keys, the keys egui and a widget
/// group move the focus with.
fn moves_focus(event: &Event) -> bool {
    let Event::Key {
        key,
        pressed: true,
        modifiers,
        physical_key: _,
        repeat: _,
    } = event
    else {
        return false;
    };

    let arrow = matches!(
        key,
        Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight
    );

    if *key == Key::Tab {
        !modifiers.any() || modifiers.shift_only()
    } else {
        arrow && !modifiers.any()
    }
}

fn from_keys(ui: &Ui, id: Id) -> bool {
    let keyboard = ui.data(|data| data.get_temp::<Keyboard>(keyboard_id()).unwrap_or_default());
    keyboard.moving || keyboard.holder == Some(id)
}

fn keyboard_id() -> Id {
    Id::unique("toolbox_gui_kit::focus::keyboard")
}
