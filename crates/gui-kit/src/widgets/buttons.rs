//! Buttons in four strengths and the frameless icon button.
//!
//! One rule for every page, so a button's weight tells what it does:
//!
//! - Soft is the default.
//! - One Primary per view at most: the action the view leads to, such as
//!   Apply. Equal peers stay Soft. A dialog is a view of its own.
//! - Danger for what cannot be taken back. KDE has no red button: it is a
//!   Breeze button, and the destructive icon and the question say what it
//!   does. So Danger draws as Soft. When a question guards it, the button
//!   that opens the question stays Soft and the question's answer is Danger.
//! - Ghost has no fill at rest and reads as a label, so pages do not use
//!   it for a lone action.
//! - On every row, item rows (a file, a driver, a user) and task rows (a
//!   maintenance job, a next step) alike, actions sit at the right,
//!   centred on the row. When that leaves the words less than
//!   [`MIN_TEXT_WIDTH`](super::layout::MIN_TEXT_WIDTH) (about 30
//!   characters), they drop under the words at the left. `split_row` and
//!   `ActionRow` do this.
//!
//! Push and flat icon buttons follow Darkly 0.5.45: a 1 px outline on a
//! raised face, the face inset 2 px, the focus as the fill, and the sizes of
//! `pushButtonSizeFromContents` (darklystyle.cpp).

use egui::{
    Color32, CornerRadius, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, WidgetInfo, pos2,
    vec2,
};

mod flat;
pub(super) mod look;

pub(super) use flat::paint_small_button;
pub use flat::{
    icon_button, icon_button_padding, icon_button_width, refresh_button, refresh_button_busy,
    sidebar_button,
};
use look::{press_overlay, push_fill, push_shadow, push_width, tool_fill};

use super::focus::focus_from_keyboard;
use super::line;
use super::press::{Pointer, faded, hover_amount};
use super::top_light::paint_button_top_light;
use crate::fonts::{BODY, Weight};
use crate::icons::Icon;
use crate::theme::CONTROL_RADIUS;
use crate::tokens::{self, Tokens};

/// Darkly's `Frame_FrameWidth`, the room round a push button's face.
const FRAME: f32 = 2.0;
/// Darkly's `ButtonHeight` (3), the margins above and below (12) and the
/// frame on both sides (4), one pixel less than their sum of 19, so the
/// height stays even on the 16 px text.
pub(super) const BUTTON_EXTRA_HEIGHT: f32 = 18.0;
const ICON_SIZE: f32 = 16.0;
/// Darkly's `Button_ItemSpacing`, between a push button's icon and its text.
const ITEM_SPACING: f32 = 4.0;
/// A glyph whose picture has no size of its own.
pub const GLYPH: f32 = 18.0;
pub(super) const SMALL_BUTTON: f32 = 24.0;
pub const SMALL_GLYPH: f32 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    /// Other actions, on a field fill.
    Soft,
    /// Actions beside a primary one, without a fill until hovered.
    Ghost,
    Danger,
}

/// What a raised face shows. `sunken` draws it pressed, as a press or an
/// open dropdown list does. The ripple follows the pointer's own press.
#[derive(Clone, Copy, Debug)]
pub(super) struct FaceState {
    pub(super) sunken: bool,
    pub(super) pointer: Pointer,
    pub(super) focused: bool,
}

impl ButtonKind {
    /// The fill a button rests on: Darkly's button colour, or the default
    /// button's grey, which takes the place of the accent.
    const fn base(self, tokens: &Tokens) -> Color32 {
        match self {
            Self::Primary => tokens.default_button,
            Self::Soft | Self::Danger => tokens.input,
            Self::Ghost => Color32::TRANSPARENT,
        }
    }

    pub(super) fn fill(self, tokens: &Tokens, pointer: Pointer) -> Color32 {
        self.focused_fill(tokens, pointer, false)
    }

    fn focused_fill(self, tokens: &Tokens, pointer: Pointer, focused: bool) -> Color32 {
        match self {
            // Darkly's flat tool button: the text colour under the pointer,
            // the focus colour when only the focus holds it.
            Self::Ghost => tool_fill(tokens, pointer, focused).unwrap_or(Color32::TRANSPARENT),
            Self::Primary | Self::Soft | Self::Danger => {
                push_fill(tokens, self.base(tokens), pointer, focused)
            }
        }
    }

    /// The fill at rest, for tests that tell the kinds apart by it. Danger
    /// shares Soft's fill, as KDE has no red button.
    pub fn rest_fill(self, tokens: &Tokens) -> Color32 {
        self.fill(tokens, Pointer::Rest)
    }

    /// Whether the fill under the pointer is the keyboard focus fill, which
    /// the text reads on. A flat button under the pointer takes the text
    /// colour's tint instead, and a press keeps the pressed fill.
    pub(super) fn shows_focus_fill(self, pointer: Pointer, focused: bool) -> bool {
        focused
            && match self {
                Self::Ghost => pointer == Pointer::Rest,
                Self::Primary | Self::Soft | Self::Danger => pointer != Pointer::Pressed,
            }
    }

    const fn raised(self) -> bool {
        matches!(self, Self::Primary | Self::Soft | Self::Danger)
    }

    /// A press shows at once, as the finger expects it.
    pub(super) fn now(self, ui: &Ui, response: &Response, tokens: &Tokens) -> Color32 {
        let pointer = Pointer::of(response);
        let focused = focus_from_keyboard(ui, response) && ui.is_enabled();
        if pointer == Pointer::Pressed {
            return self.focused_fill(tokens, pointer, focused);
        }

        let hover = hover_amount(ui, response, pointer);
        self.focused_fill(tokens, Pointer::Rest, focused)
            .lerp_to_gamma(self.focused_fill(tokens, Pointer::Hovered, focused), hover)
    }
}

pub fn pill_button(ui: &mut Ui, text: &str, kind: ButtonKind) -> Response {
    pill(ui, text, kind, 0.0, None)
}

/// Screen readers hear `name`, for a row of buttons that would all say "Edit" or "Wiki".
pub fn named_pill_button(ui: &mut Ui, text: &str, kind: ButtonKind, name: &str) -> Response {
    pill(ui, text, kind, 0.0, None).accessible_name(name)
}

/// For a button whose text changes (Review and Hide), so what sits beside it stays put.
pub fn pill_button_at_least(ui: &mut Ui, text: &str, kind: ButtonKind, width: f32) -> Response {
    pill(ui, text, kind, width, None)
}

/// A button with its standard icon before the text, as KDE's dialog
/// buttons show theirs (`ButtonsHaveIcons`).
pub fn pill_button_with_icon(ui: &mut Ui, text: &str, kind: ButtonKind, icon: Icon) -> Response {
    pill(ui, text, kind, 0.0, Some(icon))
}

/// [`pill_button_with_icon`] at least `width` wide, for a text that changes.
pub fn pill_button_with_icon_at_least(
    ui: &mut Ui,
    text: &str,
    kind: ButtonKind,
    width: f32,
    icon: Icon,
) -> Response {
    pill(ui, text, kind, width, Some(icon))
}

/// How wide [`pill_button_with_icon`] draws `text`, whichever icon it has, for
/// a layout that places it before drawing.
pub fn pill_width_with_icon(ui: &Ui, text: &str) -> f32 {
    let galley = line(ui, text, Weight::Regular, BODY, Color32::PLACEHOLDER);
    push_width(galley.size().x + icon_run(), 0.0)
}

/// How far an icon pushes the text right: the icon and the gap after it.
const fn icon_run() -> f32 {
    ICON_SIZE + ITEM_SPACING
}

fn pill(ui: &mut Ui, text: &str, kind: ButtonKind, width: f32, icon: Option<Icon>) -> Response {
    let tokens = tokens::current(ui.ctx());
    let galley = line(ui, text, Weight::Regular, BODY, Color32::PLACEHOLDER);
    let run = if icon.is_some() { icon_run() } else { 0.0 };
    let size = vec2(push_width(galley.size().x + run, width), push_height(ui));

    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), text));

    if ui.is_rect_visible(rect) {
        // A disabled Ui fades the button's own colours, the rule in
        // `press.rs`, so a Ghost stays without a fill.
        let pointer = Pointer::of(&response);
        let focused = focus_from_keyboard(ui, &response) && ui.is_enabled();
        let fill = kind.now(ui, &response, &tokens);
        let ink = if kind.shows_focus_fill(pointer, focused) {
            tokens.on_focus
        } else {
            // Darkly draws every push button's text in the text colour,
            // also on the accent and the danger fill.
            tokens.text
        };

        let face = rect.shrink(FRAME);
        let face = if pointer == Pointer::Pressed {
            face.translate(vec2(0.0, 1.0))
        } else {
            face
        };

        if kind.raised() {
            let state = FaceState {
                sunken: pointer == Pointer::Pressed,
                pointer,
                focused,
            };

            paint_push_face(ui, face, fill, state);
        } else {
            ui.painter()
                .rect_filled(face, CornerRadius::same(CONTROL_RADIUS), fill);
        }

        // The icon and the text are centred together, as Breeze lays them out.
        let content = galley.size().x + run;
        let left = content.mul_add(-0.5, rect.center().x);
        if let Some(icon) = icon {
            let glyph = Rect::from_center_size(
                pos2(left + ICON_SIZE / 2.0, rect.center().y),
                vec2(ICON_SIZE, ICON_SIZE),
            );

            let theme = crate::icons::theme_in_use(ui.ctx());
            icon.image(theme.as_deref(), ink)
                .fit_to_exact_size(vec2(ICON_SIZE, ICON_SIZE))
                .paint_at(ui, glyph);
        }

        let position = pos2(left + run, rect.center().y - galley.size().y / 2.0);
        ui.painter().galley(position, galley, ink);
    }

    response
}

/// The face of a raised push button or a closed dropdown, `face` being
/// the widget less its frame, filled with `fill`: its shadow, the card
/// outline at rest and under the pointer, the top light and the ripple.
pub(super) fn paint_push_face(ui: &Ui, face: Rect, fill: Color32, state: FaceState) {
    let tokens = tokens::current(ui.ctx());
    let painter = ui.painter();
    let enabled = ui.is_enabled();
    let shadow_pointer = if state.sunken {
        Pointer::Pressed
    } else {
        state.pointer
    };

    if enabled {
        let shadow = push_shadow(&tokens, shadow_pointer, state.focused);
        painter.add(shadow.as_shape(face, CONTROL_RADIUS));
    }

    painter.rect_filled(face, CornerRadius::same(CONTROL_RADIUS), fill);

    // The edge a field draws too. A focus or press fill replaces it.
    if !state.focused && !state.sunken {
        painter.rect_stroke(
            face,
            CornerRadius::same(CONTROL_RADIUS),
            Stroke::new(1.0, faded(ui, tokens.card_outline, enabled)),
            StrokeKind::Inside,
        );
    }

    if enabled {
        paint_button_top_light(painter, face, CONTROL_RADIUS);
    }

    if state.pointer == Pointer::Pressed {
        painter.rect_filled(face, CONTROL_RADIUS, press_overlay(&tokens));
    }
}

/// A push button's height in `ui`: its text plus 18 px. A line that holds
/// one grows to it (`text_block_height_with`).
pub fn push_height(ui: &Ui) -> f32 {
    super::list::text_line_height(ui, Weight::Regular, BODY) + BUTTON_EXTRA_HEIGHT
}

/// How wide [`pill_button`] draws `text`, for a layout that places it
/// before drawing.
pub fn pill_width(ui: &Ui, text: &str) -> f32 {
    let galley = line(ui, text, Weight::Regular, BODY, Color32::PLACEHOLDER);
    push_width(galley.size().x, 0.0)
}

/// One row of pill buttons at the right end of the space. `widths` are the
/// buttons `add` draws, as [`pill_width`] or [`pill_width_with_icon`] gives
/// them, so the row knows its width before it draws them.
pub fn right_buttons<R>(ui: &mut Ui, widths: &[f32], add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.horizontal(|ui| {
        let gap = tokens::current(ui.ctx()).sizes.control_gap;
        ui.spacing_mut().item_spacing.x = gap;
        let width = widths.iter().map(|width| width + gap).sum::<f32>() - gap;

        ui.add_space((ui.available_width() - width).max(0.0));
        add(ui)
    })
    .inner
}
