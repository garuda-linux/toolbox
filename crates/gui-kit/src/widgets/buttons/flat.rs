//! Flat icon buttons: the frameless glyph buttons of a header, a row and a
//! field, drawn as Darkly's tool buttons.

use egui::{Color32, CornerRadius, Rect, Response, Role, Sense, Spinner, Ui, WidgetInfo, vec2};

use super::super::Choice;
use super::super::focus::focus_from_keyboard;
use super::super::press::Pointer;
use super::SMALL_GLYPH;
use super::look::tool_fill;
use crate::icons::Icon;
use crate::theme::TOOL_BUTTON_RADIUS;
use crate::tokens::{self, Tokens};
use crate::ui_ext::icon_image;

/// Darkly's `ToolButton_MarginWidth`, all round a flat icon button, so a
/// 16 px icon makes a 28 px square.
const ICON_PADDING: f32 = 6.0;
/// A flat button's face sits this far inside its widget (darklyhelper.cpp:896).
const FLAT_INSET: f32 = 1.0;

/// How far an [`icon_button`]'s glyph sits inside its rect, for a row that
/// places it by what it paints.
pub const fn icon_button_padding() -> f32 {
    ICON_PADDING
}

/// How wide an [`icon_button`] is, for a row that makes room for one.
pub const fn icon_button_width() -> f32 {
    ICON_PADDING + ICON_PADDING + SMALL_GLYPH
}

/// `name` names it and shows on hover.
pub fn icon_button(ui: &mut Ui, icon: Icon, name: &str) -> Response {
    let idle = tokens::current(ui.ctx()).secondary;
    glyph_button(ui, icon, name, idle, false).on_hover_text(name)
}

pub fn sidebar_button(ui: &mut Ui, icon: Icon, name: &str) -> Response {
    let idle = tokens::current(ui.ctx()).sidebar_text;
    glyph_button(ui, icon, name, idle, false).on_hover_text(name)
}

fn glyph_button(ui: &mut Ui, icon: Icon, name: &str, idle: Color32, spinning: bool) -> Response {
    let glyph = vec2(SMALL_GLYPH, SMALL_GLYPH);
    let size = glyph + vec2(ICON_PADDING, ICON_PADDING) * 2.0;
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), name));

    if ui.is_rect_visible(rect) {
        let tint = paint_face(ui, rect, &response, idle);

        let glyph_rect = Rect::from_center_size(rect.center(), glyph);
        if spinning {
            Spinner::new()
                .size(glyph.x)
                .color(tint)
                .paint_at(ui, glyph_rect);
        } else {
            icon_image(ui, icon, tint)
                .fit_to_exact_size(glyph)
                .paint_at(ui, glyph_rect);
        }
    }

    response
}

/// The face of a flat button, inside its widget and rounded as Darkly's
/// tool button (`renderToolButtonFrame`).
fn paint_flat(ui: &Ui, rect: Rect, fill: Option<Color32>) {
    if let Some(fill) = fill {
        ui.painter().rect_filled(
            rect.shrink(FLAT_INSET),
            CornerRadius::same(TOOL_BUTTON_RADIUS),
            fill,
        );
    }
}

fn paint_face(ui: &Ui, rect: Rect, response: &Response, idle: Color32) -> Color32 {
    let tokens = tokens::current(ui.ctx());
    let pointer = Pointer::of(response);
    let focused = focus_from_keyboard(ui, response) && ui.is_enabled();
    paint_flat(ui, rect, tool_fill(&tokens, pointer, focused));
    flat_tint(&tokens, pointer, focused, idle)
}

/// The glyph colour: the focus colour when only the focus holds it, the text
/// colour under the pointer or with the focus, `idle` otherwise.
fn flat_tint(tokens: &Tokens, pointer: Pointer, focused: bool, idle: Color32) -> Color32 {
    if focused && pointer == Pointer::Rest {
        tokens.on_focus
    } else if pointer.is_under() || focused {
        tokens.text
    } else {
        idle
    }
}

/// F5 presses it too, so a page is read again from the keyboard alone.
/// F5 is left to a dialog over the page and to the program in a focused
/// terminal.
pub fn refresh_button(ui: &mut Ui, name: &str) -> Choice<()> {
    refresh_button_busy(ui, name, None)
}

/// [`refresh_button`] that, while `busy` names the read in progress, spins,
/// is named `busy` and takes neither a click nor F5, so no second read
/// starts. It stays enabled, so it keeps the focus.
pub fn refresh_button_busy(ui: &mut Ui, name: &str, busy: Option<&str>) -> Choice<()> {
    let spinning = busy.is_some();
    let name = busy.unwrap_or(name);
    let idle = tokens::current(ui.ctx()).secondary;
    let button =
        glyph_button(ui, crate::icons::VIEW_REFRESH, name, idle, spinning).on_hover_text(name);

    let clicked = button.clicked() && !spinning;
    let ctx = ui.ctx().clone();
    let free = !spinning
        && ui.is_enabled()
        && ctx.memory(egui::Memory::top_modal_layer).is_none()
        && !crate::terminal_view::owns_keys(&ctx);

    let pressed =
        free && ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::F5));

    Choice {
        response: button,
        value: (clicked || pressed).then_some(()),
    }
}

/// A small flat icon button, such as a field's or a row's, in Darkly's
/// tool button look.
pub(in crate::widgets) fn paint_small_button(ui: &Ui, rect: Rect, response: &Response, icon: Icon) {
    let idle = tokens::current(ui.ctx()).secondary;
    let tint = paint_face(ui, rect, response, idle);
    let glyph = Rect::from_center_size(rect.center(), vec2(SMALL_GLYPH, SMALL_GLYPH));
    icon_image(ui, icon, tint).paint_at(ui, glyph);
}
