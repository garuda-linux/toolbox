use egui::{Color32, Response, Role, Sense, Ui, UiBuilder, WidgetInfo, vec2};

use super::press::Pointer;
use super::{focus_ring, line};
use crate::fonts::{SMALL, Weight};
use crate::tokens::{self, Tokens};

const CHIP_HEIGHT: f32 = 32.0;
const CHIP_RADIUS: u8 = 16;
const CHIP_PADDING_X: f32 = 14.0;
const CHIP_GAP: f32 = 7.0;
const TAG_HEIGHT: f32 = 22.0;
const TAG_RADIUS: u8 = 11;
const TAG_PADDING_X: f32 = 10.0;
pub(super) const TAG_SCALE: f32 = SMALL * 0.92;
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
    /// Neutral is the secondary colour, so a fact stays quiet.
    const fn dot(self, tokens: &Tokens) -> Color32 {
        match self {
            Self::Accent => tokens.accent,
            Self::Neutral => tokens.secondary,
            Self::Warning => tokens.warning,
            Self::Success => tokens.success,
            Self::Error => tokens.error,
        }
    }

    pub(super) fn colors(self, tokens: &Tokens) -> (Color32, Color32) {
        match self {
            Self::Accent => (tokens.accent_soft, tokens.accent),
            Self::Neutral => (tokens.hover, tokens.secondary),
            Self::Warning => (tokens.hover, tokens.warning),
            Self::Success => (tokens.tag_wash(tokens.success), tokens.success),
            Self::Error => (tokens.tag_wash(tokens.error), tokens.error),
        }
    }
}

/// A selected chip takes the accent tint, as the sidebar's entry does.
pub fn chip(ui: &mut Ui, label: &str, count: Option<&str>, selected: bool) -> Response {
    let tokens = tokens::current(ui.ctx());
    let (fill, ink, muted) = if selected {
        (tokens.accent_tint, tokens.on_tint, tokens.on_tint)
    } else {
        (tokens.field, tokens.text, tokens.secondary)
    };

    let name = line(ui, label, Weight::Medium, SMALL, ink);
    let number = count.map(|count| line(ui, count, Weight::Medium, SMALL, muted));
    let number_width = number
        .as_ref()
        .map_or(0.0, |number| number.size().x + CHIP_GAP);

    let size = vec2(
        CHIP_PADDING_X.mul_add(2.0, name.size().x + number_width),
        CHIP_HEIGHT,
    );

    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let spoken = count.map_or_else(|| label.to_owned(), |count| format!("{label} {count}"));
    response.widget_info(|| WidgetInfo::selected(Role::Button, ui.is_enabled(), selected, &spoken));

    if ui.is_rect_visible(rect) {
        let fill = chip_fill(&tokens, fill, selected, Pointer::of(&response));

        ui.painter().rect_filled(rect, CHIP_RADIUS, fill);

        let top = rect.center().y - name.size().y / 2.0;
        let left = rect.left() + CHIP_PADDING_X;
        let name_width = name.size().x;
        ui.painter().galley(egui::pos2(left, top), name, ink);
        if let Some(number) = number {
            let at = egui::pos2(left + name_width + CHIP_GAP, top);
            ui.painter().galley(at, number, muted);
        }

        focus_ring(ui, &response, CHIP_RADIUS);
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

/// Used for a state every row of a list shows, where a coloured pill would repeat down the page.
pub fn status_dot(ui: &mut Ui, label: &str, tone: Tone) -> Response {
    let tokens = tokens::current(ui.ctx());
    let text = line(ui, label, Weight::Medium, SMALL, tokens.secondary);
    let height = text.size().y.max(DOT);
    let (rect, response) =
        ui.allocate_exact_size(vec2(DOT + DOT_GAP + text.size().x, height), Sense::hover());

    response.widget_info(|| WidgetInfo::labeled(Role::Label, true, label));

    if ui.is_rect_visible(rect) {
        let centre = egui::pos2(rect.left() + DOT / 2.0, rect.center().y);
        ui.painter()
            .circle_filled(centre, DOT / 2.0, tone.dot(&tokens));
        let at = egui::pos2(
            rect.left() + DOT + DOT_GAP,
            rect.center().y - text.size().y / 2.0,
        );

        ui.painter().galley(at, text, tokens.secondary);
    }

    response
}

/// A chip's fill: a hover shows on an unselected chip only, a press on
/// either, so a selected chip still answers the finger.
pub(super) const fn chip_fill(
    tokens: &Tokens,
    rest: Color32,
    selected: bool,
    pointer: Pointer,
) -> Color32 {
    match (pointer, selected) {
        (Pointer::Pressed, false) => tokens.pressed,
        (Pointer::Pressed, true) => tokens.tint_pressed,
        (Pointer::Hovered, false) => tokens.hover,
        (Pointer::Hovered, true) | (Pointer::Rest, _) => rest,
    }
}

/// Returns the option clicked this frame.
pub fn chips<T>(ui: &mut Ui, options: &[(T, &str)], current: T) -> Option<T>
where
    T: PartialEq + Copy,
{
    counted_chips(ui, &uncounted(options), current)
}

/// [`chips`] in a group that `name` names, for a page with more than one
/// row of chips: each row's "All" then tells which row it belongs to.
pub fn named_chips<T>(ui: &mut Ui, name: &str, options: &[(T, &str)], current: T) -> Option<T>
where
    T: PartialEq + Copy,
{
    let builder = UiBuilder::new()
        .id_salt(("chips", name))
        .accessibility_role(Role::Group)
        .accessibility_label(name);

    ui.scope_builder(builder, |ui| chips(ui, options, current))
        .inner
}

pub fn counted_chips<T>(ui: &mut Ui, options: &[(T, &str, Option<&str>)], current: T) -> Option<T>
where
    T: PartialEq + Copy,
{
    chip_row(ui, options, |value| value == current)
}

/// Returns the option clicked this frame.
pub fn toggle_chips<T>(ui: &mut Ui, options: &[(T, &str)], on: impl Fn(T) -> bool) -> Option<T>
where
    T: Copy,
{
    chip_row(ui, &uncounted(options), on)
}

fn uncounted<'a, T>(options: &[(T, &'a str)]) -> Vec<(T, &'a str, Option<&'a str>)>
where
    T: Copy,
{
    options
        .iter()
        .map(|(value, label)| (*value, *label, None))
        .collect()
}

fn chip_row<T>(
    ui: &mut Ui,
    options: &[(T, &str, Option<&str>)],
    selected: impl Fn(T) -> bool,
) -> Option<T>
where
    T: Copy,
{
    let gap = tokens::current(ui.ctx()).sizes.control_gap;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(gap, gap);
        let mut clicked = None;
        for (value, label, count) in options {
            if chip(ui, label, *count, selected(*value)).clicked() {
                clicked = Some(*value);
            }
        }

        clicked
    })
    .inner
}

pub fn tag(ui: &mut Ui, text: &str, tone: Tone) -> Response {
    let tokens = tokens::current(ui.ctx());
    painted_tag(ui, text, Sense::hover(), Role::Label, |_| {
        tone.colors(&tokens)
    })
}

/// The ink reads 4.5:1 under the pointer and when pressed, on a selected chip's accent tint.
pub fn link_tag(ui: &mut Ui, text: &str, tone: Tone) -> Response {
    let tokens = tokens::current(ui.ctx());
    let response = painted_tag(
        ui,
        text,
        Sense::click(),
        Role::Link,
        |response| match Pointer::of(response) {
            Pointer::Rest => tone.colors(&tokens),
            Pointer::Hovered => (tokens.accent_tint, tokens.on_tint),
            Pointer::Pressed => (tokens.tint_pressed, tokens.on_tint),
        },
    );

    focus_ring(ui, &response, TAG_RADIUS);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
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
    let (_, own_baseline) = capital(Weight::SemiBold, TAG_SCALE);
    middle - beside_height / 2.0 + beside_baseline - own_baseline
}

fn painted_tag(
    ui: &mut Ui,
    text: &str,
    sense: Sense,
    role: Role,
    colours: impl FnOnce(&Response) -> (Color32, Color32),
) -> Response {
    // The ink waits for the pointer, so the line is measured before it
    // is laid out in its colour. egui keeps both layouts cached.
    let measured = line(ui, text, Weight::SemiBold, TAG_SCALE, Color32::PLACEHOLDER);
    let width = TAG_PADDING_X.mul_add(2.0, measured.size().x);

    // The pill sits on the line's middle, as a status dot's words do. On
    // a shorter line it takes only the line's height, since growing the
    // line would move the middle of what is already on it.
    let slot = TAG_HEIGHT.min(ui.spacing().interact_size.y);
    let (id, taken) = ui.allocate_space(vec2(width, slot));
    let rect = egui::Rect::from_center_size(taken.center(), vec2(width, TAG_HEIGHT));

    let response = ui.interact(rect, id, sense);
    response.widget_info(|| WidgetInfo::labeled(role, ui.is_enabled(), text));

    if ui.is_rect_visible(rect) {
        let (fill, ink) = colours(&response);
        let galley = line(ui, text, Weight::SemiBold, TAG_SCALE, ink);
        ui.painter().rect_filled(rect, TAG_RADIUS, fill);
        let left = rect.center().x - galley.size().x / 2.0;
        let position = egui::pos2(left, words_top(ui, rect.center().y));
        ui.painter().galley(position, galley, ink);
    }

    response
}
