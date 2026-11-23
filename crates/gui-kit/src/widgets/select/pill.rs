//! The closed pill of a dropdown, painted as Darkly's push button, with
//! its chevron.

use std::f32::consts::PI;

use egui::{Color32, Event, Rect, Response, Ui, Vec2, pos2, vec2};

use super::super::buttons::look::push_shadow;
use super::super::focus::focus_from_keyboard;
use super::super::press::{Pointer, hover_amount};
use super::super::rich;
use super::super::text::cut;
use super::options::{SWATCH_GAP, SWATCH_WIDTH, paint_swatch};
use crate::fonts::{BODY, Weight};
use crate::motion::{self, Timing};
use crate::theme::CONTROL_RADIUS;
use crate::tokens::{Tokens, darkly, mix};
use crate::ui_ext::icon_image;
use crate::{icons, tokens};

/// The label's inset from the pill's end, 8 px as the push button draws
/// its label (Darkly's `ComboBox_FrameWidth` is 7, with the
/// shadow's room outside).
pub(super) const TEXT_INSET: f32 = 8.0;
/// The chevron's centre from the pill's end: Darkly's arrow sits in the
/// frame's inset, 3 px left of its own square (darklystyle.cpp:3145-3152).
pub(super) const CHEVRON_INSET: f32 = 14.0;
pub(super) const CHEVRON: f32 = 16.0;
/// Darkly's `Frame_FrameWidth`, the room the button keeps for its shadow.
pub const FRAME: f32 = 2.0;
/// How far the arrow moves from the text toward the view colour, as Darkly's
/// combo arrow mixes it (`mix(text, base, 0.15)`).
const ARROW_SHADE: f32 = 0.15;
/// Draws the pill as Darkly's push button, see [`button_fill`]. `sunken`
/// is whether the list is open, which Darkly draws as pressed. Returns
/// whether `shown` was cut to fit.
pub(super) fn paint_pill(
    ui: &Ui,
    response: &Response,
    rect: Rect,
    shown: &str,
    (swatch, sunken): (Option<[Color32; 3]>, bool),
) -> bool {
    if !ui.is_rect_visible(rect) {
        return false;
    }

    let tokens = tokens::current(ui.ctx());
    let frame = rect.shrink(FRAME);
    let pointer = Pointer::of(response);
    let lit = focus_from_keyboard(ui, response) || sunken;

    // An open box shadows as a pressed one, as Darkly draws it.
    if ui.is_enabled() {
        let shadow_pointer = if sunken { Pointer::Pressed } else { pointer };
        let shadow = push_shadow(&tokens, shadow_pointer, lit);
        ui.painter().add(shadow.as_shape(frame, CONTROL_RADIUS));
    }

    let fill = button_fill(ui, response, pointer, sunken, &tokens);
    ui.painter().rect_filled(frame, CONTROL_RADIUS, fill);

    let before = swatch.map_or(0.0, |colours| {
        paint_swatch(ui, rect.left() + TEXT_INSET, rect.center().y, colours);
        SWATCH_WIDTH + SWATCH_GAP
    });

    // Darkly's label takes the highlighted text while the box holds the
    // focus or is open (darklystyle.cpp:5511-5527).
    let ink = if focus_from_keyboard(ui, response) && !sunken && pointer != Pointer::Pressed {
        tokens.on_focus
    } else if lit {
        tokens.highlighted_text
    } else {
        tokens.text
    };

    let text = Rect::from_min_max(
        pos2(rect.left() + TEXT_INSET + before, rect.top()),
        pos2(
            rect.right() - CHEVRON_INSET - CHEVRON / 2.0 - tokens.sizes.text_to_icon,
            rect.bottom(),
        ),
    );

    let galley = cut(
        ui,
        rich(ui, shown, Weight::Medium, BODY, ink),
        text.width().max(0.0),
    );

    let was_cut = galley.elided;
    let text_top = rect.center().y - galley.size().y / 2.0;
    ui.painter()
        .with_clip_rect(text)
        .galley(pos2(text.left(), text_top), galley, ink);

    was_cut
}

/// Darkly's `buttonBackgroundColor` and `renderButtonFrame` for a combo
/// box (darklyhelper.cpp:260-282, 811-850): the Button colour at rest, the
/// focus colour after keyboard focus only, lighter under the pointer, and
/// the pressed colour while pressed or open. The focus colour lightens by
/// 15 % and then 2 % under the pointer.
fn button_fill(
    ui: &Ui,
    response: &Response,
    pointer: Pointer,
    sunken: bool,
    tokens: &Tokens,
) -> Color32 {
    if pointer == Pointer::Pressed || sunken {
        return tokens.button_pressed;
    }

    let (rest, hovered) = if focus_from_keyboard(ui, response) {
        let focus = tokens.focus_fill;
        (focus, darkly::lighter(darkly::lighter(focus, 1.15), 1.02))
    } else {
        (tokens.field, tokens.button_hover)
    };

    rest.lerp_to_gamma(hovered, hover_amount(ui, response, pointer))
}

pub(super) fn paint_chevron(ui: &Ui, pill: &Response, open: bool) {
    let turn = Timing::both(motion::CHEVRON_TURN);
    let id = pill.id.with("chevron");

    // A key that opened or closed the list jumps, a click turns. A key
    // pressed while a click's turn runs leaves that turn alone.
    let was_open = ui.data_mut(|data| {
        let key = id.with("open");
        let was = data.get_temp::<bool>(key);
        data.insert_temp(key, open);
        was
    });

    let toggled = was_open.is_some_and(|was| was != open);
    let keyed = toggled
        && ui.input(|input| {
            input.events.iter().any(|event| {
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
            })
        });

    if keyed {
        motion::settle(ui.ctx(), id, open);
    }

    let turned = motion::presence(ui.ctx(), id, open, turn);
    let rect = pill.rect;
    if !ui.is_rect_visible(rect) {
        return;
    }

    let chevron = Rect::from_center_size(
        pos2(rect.right() - CHEVRON_INSET, rect.center().y),
        vec2(CHEVRON, CHEVRON),
    );

    // Darkly's arrow of a closed box, shaded toward the button, takes the
    // highlighted text while the box holds the focus or is open.
    let tokens = tokens::current(ui.ctx());
    let ink = if focus_from_keyboard(ui, pill) && !open {
        tokens.on_focus
    } else if open {
        tokens.highlighted_text
    } else {
        mix(tokens.text, tokens.view, ARROW_SHADE)
    };

    let image = icon_image(ui, icons::GO_DOWN, ink);
    let image = if turned > 0.0 {
        image.rotate(turned * PI, Vec2::splat(0.5))
    } else {
        image
    };

    image.paint_at(ui, chevron);
}
