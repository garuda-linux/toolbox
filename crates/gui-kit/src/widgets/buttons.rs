//! Pill buttons in four strengths and the frameless icon button.
//!
//! One rule for every page, so a button's weight tells what it does:
//!
//! - Soft is the default.
//! - One Primary per view at most: the action the view leads to, such as
//!   Apply. Equal peers stay Soft. A dialog is a view of its own.
//! - Danger for what cannot be taken back. When a question guards it, the
//!   button that opens the question stays Soft and the question's answer
//!   is Danger.
//! - Ghost has no fill at rest and reads as a label, so pages do not use
//!   it for a lone action.
//! - On every row, item rows (a file, a driver, a user) and task rows (a
//!   maintenance job, a next step) alike, actions sit at the right,
//!   centred on the row. When that leaves the words less than
//!   [`MIN_TEXT_WIDTH`](super::layout::MIN_TEXT_WIDTH) (about 30
//!   characters), they drop under the words at the left. `split_row` and
//!   `ActionRow` do this.

use egui::{
    Color32, CornerRadius, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, WidgetInfo, vec2,
};

use super::card::on_raised;
use super::press::{Pointer, faded};
use super::{focus_ring, line};
use crate::fonts::{BODY, Weight};
use crate::icons::Icon;
use crate::motion;
use crate::theme::{PILL_HEIGHT, PILL_RADIUS, RADIUS};
use crate::tokens::{self, Tokens};
use crate::ui_ext::icon_image;

const PILL_PADDING_X: f32 = 16.0;
const ICON_PADDING: f32 = 7.0;
/// A glyph whose picture has no size of its own.
const GLYPH: f32 = 18.0;
pub(super) const SMALL_BUTTON: f32 = 26.0;
const SMALL_GLYPH: f32 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    /// Other actions, on a field fill.
    Soft,
    /// Actions beside a primary one, without a fill until hovered.
    Ghost,
    Danger,
}

impl ButtonKind {
    pub(super) const fn fill(self, tokens: &Tokens, pointer: Pointer) -> Color32 {
        match (self, pointer) {
            (Self::Primary, Pointer::Rest) => tokens.accent,
            (Self::Primary, Pointer::Hovered) => tokens.accent_hover,
            (Self::Primary, Pointer::Pressed) => tokens.accent_pressed,
            (Self::Soft, Pointer::Rest) => tokens.field,
            (Self::Ghost, Pointer::Rest) => Color32::TRANSPARENT,
            (Self::Soft | Self::Ghost, Pointer::Hovered) => tokens.hover,
            (Self::Soft | Self::Ghost, Pointer::Pressed) => tokens.pressed,
            (Self::Danger, Pointer::Rest) => tokens.error,
            (Self::Danger, Pointer::Hovered) => tokens.error_hover,
            (Self::Danger, Pointer::Pressed) => tokens.error_pressed,
        }
    }

    const fn ink(self, tokens: &Tokens) -> Color32 {
        match self {
            Self::Primary => tokens.on_accent,
            Self::Soft | Self::Ghost => tokens.text,
            Self::Danger => tokens.on_error,
        }
    }

    /// A press shows at once, as the finger expects it.
    fn fill_now(self, ui: &Ui, response: &Response, tokens: &Tokens) -> Color32 {
        let pointer = Pointer::of(response);
        if pointer == Pointer::Pressed {
            return self.fill(tokens, pointer);
        }

        let seconds = motion::duration(ui.ctx(), motion::HOVER_FADE);
        let hover =
            ui.ctx()
                .animate_bool_with_time(response.id.with("hover"), pointer.is_under(), seconds);

        self.fill(tokens, Pointer::Rest)
            .lerp_to_gamma(self.fill(tokens, Pointer::Hovered), hover)
    }
}

pub fn pill_button(ui: &mut Ui, text: &str, kind: ButtonKind) -> Response {
    pill(ui, text, kind, false, 0.0)
}

/// Screen readers hear `name`, for a row of buttons that would all say "Edit" or "Wiki".
pub fn named_pill_button(ui: &mut Ui, text: &str, kind: ButtonKind, name: &str) -> Response {
    let response = pill(ui, text, kind, false, 0.0);
    let name = name.to_owned();
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_label(name));
    response
}

/// For a button whose text changes (Review and Hide), so what sits beside it stays put.
pub fn pill_button_at_least(ui: &mut Ui, text: &str, kind: ButtonKind, width: f32) -> Response {
    pill(ui, text, kind, false, width)
}

/// For a place whose wash is the button's own fill, as a neutral banner is.
pub(super) fn rimmed_pill_button(ui: &mut Ui, text: &str, kind: ButtonKind) -> Response {
    pill(ui, text, kind, true, 0.0)
}

fn pill(ui: &mut Ui, text: &str, kind: ButtonKind, rimmed: bool, width: f32) -> Response {
    let tokens = tokens::current(ui.ctx());

    // A Soft fill at rest matches the hover fill of the row it sits on.
    let rimmed = rimmed || (kind == ButtonKind::Soft && on_raised(ui));
    let galley = line(ui, text, Weight::SemiBold, BODY, Color32::PLACEHOLDER);
    let size = vec2(
        PILL_PADDING_X.mul_add(2.0, galley.size().x).max(width),
        PILL_HEIGHT,
    );

    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), text));

    if ui.is_rect_visible(rect) {
        let (fill, ink) = (kind.fill_now(ui, &response, &tokens), kind.ink(&tokens));

        // A disabled button of any kind turns quiet, so it reads as
        // waiting. A faded accent fill would leave its label unreadable.
        let (fill, ink) = if ui.is_enabled() {
            (fill, ink)
        } else {
            (tokens.field, tokens.secondary)
        };

        // A focused button shows the ring alone. A rim inside it would
        // draw a second outline a few points in.
        let rim = if rimmed && !response.has_focus() {
            Stroke::new(1.0, faded(tokens.rim, ui.is_enabled()))
        } else {
            Stroke::NONE
        };

        ui.painter()
            .rect(rect, PILL_RADIUS, fill, rim, StrokeKind::Inside);
        let position = rect.center() - galley.size() / 2.0;
        ui.painter().galley(position, galley, ink);
        focus_ring(ui, &response, PILL_RADIUS);
    }

    response
}

/// How wide [`pill_button`] draws `text`, for a layout that places it
/// before drawing.
pub fn pill_width(ui: &Ui, text: &str) -> f32 {
    let galley = line(ui, text, Weight::SemiBold, BODY, Color32::PLACEHOLDER);
    PILL_PADDING_X.mul_add(2.0, galley.size().x)
}

/// How wide an [`icon_button`] is, for a row that makes room for one.
pub fn icon_button_width(ui: &Ui) -> f32 {
    let glyph = glyph_size(&icon_image(
        ui,
        crate::icons::VIEW_REFRESH,
        Color32::PLACEHOLDER,
    ))
    .x;

    ICON_PADDING.mul_add(2.0, glyph)
}

/// `alt` names it and shows on hover.
pub fn icon_button(ui: &mut Ui, icon: Icon, alt: &str) -> Response {
    let idle = tokens::current(ui.ctx()).secondary;
    glyph_button(ui, icon, alt, ICON_PADDING, idle).on_hover_text(alt)
}

pub fn sidebar_button(ui: &mut Ui, icon: Icon, alt: &str) -> Response {
    let idle = tokens::current(ui.ctx()).sidebar_text;
    glyph_button(ui, icon, alt, ICON_PADDING, idle).on_hover_text(alt)
}

fn glyph_button(ui: &mut Ui, icon: Icon, alt: &str, padding: f32, idle: Color32) -> Response {
    let tokens = tokens::current(ui.ctx());
    let image = icon_image(ui, icon, idle);
    let glyph = glyph_size(&image);
    let size = glyph + vec2(padding, padding) * 2.0;
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), alt));

    if ui.is_rect_visible(rect) {
        let pointer = Pointer::of(&response);
        let hovered = pointer.is_under() || response.has_focus();
        let fill = match pointer {
            Pointer::Pressed => Some(tokens.pressed),
            Pointer::Hovered | Pointer::Rest => hovered.then_some(tokens.hover),
        };

        if let Some(fill) = fill {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(RADIUS), fill);
        }

        let tint = if hovered { tokens.text } else { idle };

        let glyph_rect = Rect::from_center_size(rect.center(), glyph);
        icon_image(ui, icon, tint).paint_at(ui, glyph_rect);
        focus_ring(ui, &response, RADIUS);
    }

    response
}

/// F5 presses it too, so a page is read again from the keyboard alone.
/// F5 is left to a dialog over the page and to the program in a focused
/// terminal.
pub fn refresh_button(ui: &mut Ui, name: &str) -> bool {
    let clicked = icon_button(ui, crate::icons::VIEW_REFRESH, name).clicked();
    let ctx = ui.ctx().clone();
    let free = ui.is_enabled()
        && ctx.memory(egui::Memory::top_modal_layer).is_none()
        && !crate::terminal_view::owns_keys(&ctx);

    let pressed =
        free && ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::F5));

    clicked || pressed
}

pub(super) fn glyph_size(image: &egui::Image<'_>) -> egui::Vec2 {
    image.size().unwrap_or_else(|| vec2(GLYPH, GLYPH))
}

pub(super) fn paint_small_button(
    ui: &Ui,
    rect: Rect,
    response: &Response,
    icon: Icon,
    lit: Color32,
) {
    let tokens = tokens::current(ui.ctx());
    let shown = response.hovered() || response.has_focus();
    if shown {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(RADIUS), lit);
    }

    let tint = if shown { tokens.text } else { tokens.secondary };
    let glyph = Rect::from_center_size(rect.center(), vec2(SMALL_GLYPH, SMALL_GLYPH));
    icon_image(ui, icon, tint).paint_at(ui, glyph);
    focus_ring(ui, response, RADIUS);
}
