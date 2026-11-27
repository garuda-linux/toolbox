mod line;

use egui::text::{LayoutJob, TextFormat};
use egui::{
    Align, Color32, Id, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, Vec2, WidgetInfo,
    pos2, vec2,
};
pub use line::{chips, counted_chips, named_chips, toggle_chips};

use super::focus::focus_from_keyboard;
use super::mark_word::mark_and_word;
use super::press::{ItemLook, Pointer};
use super::{line, rich, roving};
use crate::fonts::{self, BODY, SMALL, Weight};
use crate::tokens::{self, Tokens, mix};

/// Kirigami.Chip as the desktop style lays it out at a 10 pt font
/// (measured from a render).
const CHIP_HEIGHT: f32 = 27.0;
pub(super) const CHIP_RADIUS: u8 = 5;
const CHIP_PADDING_X: f32 = 8.0;
/// Kirigami's `smallSpacing` between the chip's parts.
const CHIP_GAP: f32 = 4.0;
const FRAME_CONTRAST: f32 = 0.2;
const CHECKED_FILL: f32 = 0.2;
const CHECKED_FRAME: f32 = 0.9;
const PRESSED_FILL: f32 = 0.3;
const PRESSED_FRAME: f32 = 0.7;
/// A tag is a Kirigami.Chip that takes no click.
pub(super) const TAG_HEIGHT: f32 = CHIP_HEIGHT;
const TAG_PADDING_X: f32 = CHIP_PADDING_X;
pub(super) const TAG_SCALE: f32 = BODY;
const DOT: f32 = 7.0;
const DOT_GAP: f32 = 7.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// Waits for Apply.
    Accent,
    /// A fact, such as what an entry needs.
    Neutral,
    Warning,
    Success,
    Error,
}

impl Tone {
    /// A neutral tag is a plain chip, a toned one a checked chip in its
    /// tone's colour.
    pub(super) fn look(self, tokens: &Tokens, pointer: Pointer) -> ItemLook {
        let tint = match self {
            Self::Neutral => return chip_look(tokens, tokens.accent, false, pointer),
            Self::Accent => tokens.accent,
            Self::Warning => tokens.warning,
            Self::Success => tokens.success,
            Self::Error => tokens.error,
        };

        chip_look(tokens, tint, true, pointer)
    }
}

/// A checked chip takes the highlight, as Kirigami's `DefaultChipBackground`.
pub fn chip(ui: &mut Ui, label: &str, count: Option<&str>, selected: bool) -> Response {
    chip_in(ui, label, count, selected, None)
}

/// A chip of a `chip_row`: `member` is its id and whether it is the
/// group's Tab stop. A chip alone takes no part in a stop.
fn chip_in(
    ui: &mut Ui,
    label: &str,
    count: Option<&str>,
    selected: bool,
    member: Option<(Id, bool)>,
) -> Response {
    let tokens = tokens::current(ui.ctx());

    // The painter of a chip does not take the fade of a disabled Ui, so the
    // chip fades its own colours.
    let (enabled, visuals) = (ui.is_enabled(), ui.visuals().clone());
    let fade = move |colour: Color32| {
        if enabled {
            colour
        } else {
            visuals.disable(colour)
        }
    };

    let ink = fade(tokens.text);
    let name = line(ui, label, Weight::Regular, BODY, ink);
    let number = count.map(|count| line(ui, count, Weight::Regular, BODY, ink));
    let number_width = number
        .as_ref()
        .map_or(0.0, |number| number.size().x + CHIP_GAP);

    let size = vec2(
        CHIP_PADDING_X.mul_add(2.0, name.size().x + number_width),
        CHIP_HEIGHT,
    );

    let (rect, response) = match member {
        None => ui.allocate_exact_size(size, Sense::click()),
        Some((id, stop)) => {
            let (_, rect) = ui.allocate_space(size);
            (rect, ui.interact(rect, id, roving::sense(stop)))
        }
    };

    let spoken = count.map_or_else(|| label.to_owned(), |count| format!("{label} {count}"));
    response.widget_info(|| WidgetInfo::selected(Role::Button, ui.is_enabled(), selected, &spoken));

    if ui.is_rect_visible(rect) {
        let focused = focus_from_keyboard(ui, &response) && ui.is_enabled();
        let look = chip_look(&tokens, tokens.accent, selected, Pointer::of(&response));
        paint_chip(ui, rect, focused_look(look, &tokens, focused), &fade);

        // The text on a focus fill is laid out again in the colour that
        // reads on it, as a galley keeps the colour it was laid out in.
        let (name, number, ink) = if focused {
            let on_focus = fade(tokens.on_focus);
            (
                line(ui, label, Weight::Regular, BODY, on_focus),
                count.map(|count| line(ui, count, Weight::Regular, BODY, on_focus)),
                on_focus,
            )
        } else {
            (name, number, ink)
        };

        let top = rect.center().y - name.size().y / 2.0;
        let left = rect.left() + CHIP_PADDING_X;
        let name_width = name.size().x;
        ui.painter().galley(pos2(left, top), name, ink);
        if let Some(number) = number {
            let at = pos2(left + name_width + CHIP_GAP, top);
            ui.painter().galley(at, number, ink);
        }
    }

    response
}

/// [`tag`], except [`Tone::Accent`], which waits for Apply, draws a
/// [`status_dot`]. Other tones stay tags.
pub fn queued_tag(ui: &mut Ui, label: &str, tone: Tone) -> Response {
    match tone {
        Tone::Accent => status_dot(ui, label, tone),
        Tone::Neutral | Tone::Warning | Tone::Success | Tone::Error => tag(ui, label, tone),
    }
}

/// A state every row of a list shows, where a coloured pill would repeat down the page.
pub fn status_dot(ui: &mut Ui, label: &str, tone: Tone) -> Response {
    let tokens = tokens::current(ui.ctx());
    let room = (ui.available_width() - DOT - DOT_GAP).max(0.0);
    let full = line(ui, label, Weight::Medium, SMALL, tokens.secondary);
    let cut = room < full.size().x;
    let text = if cut {
        let styled = rich(ui, label, Weight::Medium, SMALL, tokens.secondary);
        super::text::cut(ui, styled, room)
    } else {
        full
    };

    let dot = tone.status_colour(&tokens);
    let response = mark_and_word(
        ui,
        (Vec2::splat(DOT), DOT_GAP),
        text,
        (Role::Label, label),
        |ui, mark| {
            ui.painter().circle_filled(mark.center(), DOT / 2.0, dot);
        },
    );

    if cut {
        response.on_hover_text(label)
    } else {
        response
    }
}

/// [`status_dot`] whose word goes on with `rest` in the regular weight,
/// laid out as one text, so both parts share a baseline.
pub fn status_dot_then(ui: &mut Ui, label: &str, rest: &str, tone: Tone) -> Response {
    let tokens = tokens::current(ui.ctx());
    let format = |weight| TextFormat {
        valign: Align::Center,
        ..TextFormat::simple(fonts::font(ui.style(), weight, SMALL), tokens.secondary)
    };

    let mut job = LayoutJob::default();
    job.append(label, 0.0, format(Weight::Medium));
    job.append(&format!(" {rest}"), 0.0, format(Weight::Regular));

    let text = ui.fonts_mut(|fonts| fonts.layout_job(job));
    let name = format!("{label} {rest}");
    let dot = tone.status_colour(&tokens);
    mark_and_word(
        ui,
        (Vec2::splat(DOT), DOT_GAP),
        text,
        (Role::Label, &name),
        |ui, mark| {
            ui.painter().circle_filled(mark.center(), DOT / 2.0, dot);
        },
    )
}

/// Kirigami's `DefaultChipBackground`, with `tint` for the highlight.
pub(super) fn chip_look(
    tokens: &Tokens,
    tint: Color32,
    checked: bool,
    pointer: Pointer,
) -> ItemLook {
    let (fill, frame) = match (checked, pointer) {
        (_, Pointer::Pressed) => (PRESSED_FILL, PRESSED_FRAME),
        (true, _) => (CHECKED_FILL, CHECKED_FRAME),
        (false, Pointer::Hovered) => {
            return ItemLook {
                fill: Color32::TRANSPARENT,
                frame: tokens.accent,
            };
        }
        (false, Pointer::Rest) => {
            return ItemLook {
                fill: Color32::TRANSPARENT,
                frame: mix(tokens.window, tokens.text, FRAME_CONTRAST),
            };
        }
    };

    ItemLook {
        fill: tint.gamma_multiply(fill),
        frame: tint.gamma_multiply(frame),
    }
}

/// A chip with keyboard focus fills with the focus colour, as a focused
/// button does, and keeps its frame.
const fn focused_look(look: ItemLook, tokens: &Tokens, focused: bool) -> ItemLook {
    if focused {
        ItemLook {
            fill: tokens.focus_fill,
            frame: look.frame,
        }
    } else {
        look
    }
}

pub(super) fn paint_chip(ui: &Ui, rect: Rect, look: ItemLook, fade: &impl Fn(Color32) -> Color32) {
    ui.painter().rect(
        rect,
        CHIP_RADIUS,
        fade(look.fill),
        Stroke::new(1.0, fade(look.frame)),
        StrokeKind::Inside,
    );
}

/// A Kirigami.Chip that takes no click. A tone other than Neutral marks
/// the chip as a checked one marks it, in the tone's colour.
pub fn tag(ui: &mut Ui, text: &str, tone: Tone) -> Response {
    let tokens = tokens::current(ui.ctx());
    painted_tag(ui, text, tokens.text, |_| tone.look(&tokens, Pointer::Rest))
}

/// A [`tag`] of a neutral tone, or on a selected row, where the selection
/// fill is behind it and the words and the frame take the selection text.
pub fn tag_on_row(ui: &mut Ui, text: &str, selected: bool) -> Response {
    if !selected {
        return tag(ui, text, Tone::Neutral);
    }

    let tokens = tokens::current(ui.ctx());
    painted_tag(ui, text, tokens.on_tint, |_| ItemLook {
        fill: Color32::TRANSPARENT,
        frame: tokens.on_tint,
    })
}

/// Where a tag's words start so they share the baseline of the small
/// words a status dot centres on `middle`. The tag's smaller words
/// centred alone would sit lower.
fn words_top(ui: &Ui, middle: f32) -> f32 {
    let capital = |weight, size| {
        let galley = line(ui, "H", weight, size, Color32::PLACEHOLDER);
        (galley.size().y, galley.mesh_bounds.bottom())
    };

    let (beside_height, beside_baseline) = capital(Weight::Medium, SMALL);
    let (_, own_baseline) = capital(Weight::Regular, TAG_SCALE);
    middle - beside_height / 2.0 + beside_baseline - own_baseline
}

/// A tag drawn in `rest_ink` at rest.
fn painted_tag(
    ui: &mut Ui,
    text: &str,
    rest_ink: Color32,
    look: impl FnOnce(Pointer) -> ItemLook,
) -> Response {
    // The ink waits for the pointer, so the line is measured before it
    // is laid out in its colour. egui keeps both layouts cached.
    let measured = line(ui, text, Weight::Regular, TAG_SCALE, Color32::PLACEHOLDER);
    let width = TAG_PADDING_X.mul_add(2.0, measured.size().x);

    // The pill sits on the line's middle, as a status dot's words do. On
    // a shorter line it takes only the line's height, since growing the
    // line would move the middle of what is already on it.
    let slot = TAG_HEIGHT.min(ui.spacing().interact_size.y);
    let (id, taken) = ui.allocate_space(vec2(width, slot));
    let rect = Rect::from_center_size(taken.center(), vec2(width, TAG_HEIGHT));

    let response = ui.interact(rect, id, Sense::hover());
    response.widget_info(|| WidgetInfo::labeled(Role::Label, ui.is_enabled(), text));

    if ui.is_rect_visible(rect) {
        let tokens = tokens::current(ui.ctx());
        let focused = focus_from_keyboard(ui, &response) && ui.is_enabled();
        let words_ink = if focused { tokens.on_focus } else { rest_ink };

        let galley = line(ui, text, Weight::Regular, TAG_SCALE, words_ink);
        let look = focused_look(look(Pointer::of(&response)), &tokens, focused);
        paint_chip(ui, rect, look, &|colour| colour);
        let left = rect.center().x - galley.size().x / 2.0;
        let position = pos2(left, words_top(ui, rect.center().y));
        ui.painter().galley(position, galley, words_ink);
    }

    response
}
