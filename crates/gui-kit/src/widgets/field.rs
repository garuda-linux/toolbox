//! The one look of every text field: a filled box with control corners,
//! the search keeps a pill. A glyph before the text and a unit or a
//! button after it are atoms of the text edit, so the whole field is one
//! hit area that focuses the text.

use egui::epaint::RectShape;
use egui::layers::ShapeIdx;
use egui::text_edit::TextEditState;
use egui::{
    Align2, Atom, Context, Frame, Id, IdSalt, Margin, Rect, Response, Role, Sense, Shape, Stroke,
    StrokeKind, TextEdit, Ui, WidgetInfo, vec2,
};

use super::buttons::{SMALL_BUTTON, paint_small_button};
use super::focus::field_ring;
use super::rich;
use super::secret_field::revealed_id;
use crate::fonts::{BODY, Weight};
use crate::icons::Icon;
use crate::theme::{CONTROL_RADIUS, PILL_HEIGHT, PILL_RADIUS};
use crate::tokens;
use crate::ui_ext::icon_image;

/// Room between a field's ends and its text.
pub(super) const PADDING_X: i8 = 14;
/// How far a field under the pointer moves its fill from rest towards
/// the focused fill.
const HOVER_STEP: f32 = 0.5;

/// What a field holds around its text. A unit and a button never go
/// together.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Parts<'a> {
    pub(super) glyph: Option<Icon>,
    pub(super) unit: Option<&'a str>,
    pub(super) button: Option<FieldButton<'a>>,
    /// No fill and no padding, for a control that paints the pill
    /// around the field and more.
    pub(super) inset: bool,
    /// [`PILL_RADIUS`] corners instead of [`CONTROL_RADIUS`], for the
    /// search, so a filter reads apart from a control (L-05).
    pub(super) pill: bool,
}

/// A small button at a field's end, such as Clear, named `alt`.
#[derive(Clone, Copy, Debug)]
pub(super) struct FieldButton<'a> {
    pub(super) icon: Icon,
    pub(super) alt: &'a str,
}

/// `edit` as a filled field [`PILL_HEIGHT`] tall with [`CONTROL_RADIUS`]
/// corners. The caller keeps its width, alignment, hint and font.
pub fn text_field(ui: &mut Ui, edit: TextEdit<'_>) -> Response {
    field(ui, edit, Parts::default()).0
}

/// A [`text_field`] with `unit`, such as "s", right after its text in
/// the secondary colour, the two centred in the field as one group. The
/// unit is only painted, so the field's accessible name has to say it.
pub fn text_field_with_unit(ui: &mut Ui, edit: TextEdit<'_>, unit: &str) -> Response {
    let parts = Parts {
        unit: Some(unit),
        ..Parts::default()
    };

    field(ui, edit, parts).0
}

/// A [`text_field_with_unit`] without its own fill or padding, for a
/// control that paints the pill around it and more.
pub(super) fn inset_field_with_unit(ui: &mut Ui, edit: TextEdit<'_>, unit: &str) -> Response {
    let parts = Parts {
        unit: Some(unit),
        inset: true,
        ..Parts::default()
    };

    field(ui, edit, parts).0
}

/// Drops what egui keeps of the field `id` once it is gone.
///
/// Its undo history holds every earlier version of the text, which for a
/// password would outlive the dialog. A password shown in the dialog is
/// hidden again the next time it opens.
pub fn forget_text(ctx: &Context, id: Id) {
    if let Some(mut state) = TextEditState::load(ctx, id) {
        state.clear_undoer();
    }

    ctx.data_mut(|data| {
        data.remove::<TextEditState>(id);
        data.remove::<bool>(revealed_id(id));
    });
}

/// `edit` in a field with `parts`: the text's response, and whether the
/// button was clicked.
pub(super) fn field(ui: &mut Ui, edit: TextEdit<'_>, parts: Parts<'_>) -> (Response, bool) {
    let tokens = tokens::current(ui.ctx());
    let gap = tokens.sizes.text_to_icon;
    let padding = if parts.inset { 0 } else { PADDING_X };

    // A button keeps as much room to the end as to the top.
    let end = if parts.button.is_some() {
        whole_points((PILL_HEIGHT - SMALL_BUTTON) / 2.0)
    } else {
        padding
    };

    let margin = Margin {
        left: padding,
        right: end,
        ..Margin::default()
    };

    let mut edit = edit
        .frame(Frame::NONE.inner_margin(margin))
        .min_size(vec2(0.0, PILL_HEIGHT));

    if let Some(glyph) = parts.glyph {
        edit = edit.prefix((icon_image(ui, glyph, tokens.dim), space(gap)));
    }

    if let Some(unit) = parts.unit {
        let ink = if ui.is_enabled() {
            tokens.secondary
        } else {
            tokens.dim
        };

        // The text's atom grows by half the spare room, which goes before
        // the text, and the last atom takes the other half, so text and
        // unit centre as one group.
        let label = rich(ui, unit, Weight::Regular, BODY, ink);
        edit = edit
            .align(Align2::RIGHT_CENTER)
            .suffix((space(gap), label, Atom::grow()));
    }

    let button_salt = IdSalt::new("toolbox-field-button");
    if parts.button.is_some() {
        let room = Atom::custom(button_salt, vec2(SMALL_BUTTON, SMALL_BUTTON));
        edit = edit.suffix((space(tokens.sizes.tight_gap), room));
    }

    // The fill goes under the text but depends on the focus the field
    // only reports once it is drawn.
    let slot = ui.painter().add(Shape::Noop);

    // The parts bring their own gaps, egui's would come on top.
    let spacing = ui.spacing().icon_spacing;
    ui.spacing_mut().icon_spacing = 0.0;
    let output = edit.show(ui);
    ui.spacing_mut().icon_spacing = spacing;

    let button_rect = output.response.rect(button_salt);
    let response = output.response.response;
    let clicked = parts
        .button
        .zip(button_rect)
        .is_some_and(|(button, rect)| field_button(ui, response.id, rect, button));

    if !parts.inset {
        let radius = if parts.pill {
            PILL_RADIUS
        } else {
            CONTROL_RADIUS
        };

        paint_field(ui, slot, response.rect, radius, response.has_focus());
    }

    (response, clicked)
}

/// An empty atom `width` wide.
fn space(width: f32) -> Atom<'static> {
    Atom {
        size: Some(vec2(width, 0.0)),
        ..Atom::default()
    }
}

/// Added after the text edit, so it lies over it and takes the click.
fn field_button(ui: &Ui, field: Id, rect: Rect, button: FieldButton<'_>) -> bool {
    let response = ui.interact(rect, field.with("button"), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), button.alt));
    let pressed = tokens::current(ui.ctx()).pressed;
    paint_small_button(ui, rect, &response, button.icon, pressed);
    response.on_hover_text(button.alt).clicked()
}

/// `points` rounded up to whole points, as a margin holds them.
fn whole_points(points: f32) -> i8 {
    (0..=i8::MAX)
        .find(|&whole| f32::from(whole) >= points)
        .unwrap_or(i8::MAX)
}

/// A field's fill in `slot` on `field` with `radius` corners, a step
/// lighter under the pointer and a full step while it holds the focus.
/// No rim: the fill alone marks the field, decided against the 3:1 edge a
/// rim gave (Nico, 2026-11-11).
pub(super) fn paint_field(ui: &Ui, slot: ShapeIdx, field: Rect, radius: u8, focused: bool) {
    let tokens = tokens::current(ui.ctx());
    let hovered = ui.is_enabled() && ui.rect_contains_pointer(field);
    let fill = if focused {
        tokens.hover
    } else if hovered {
        tokens.field.lerp_to_gamma(tokens.hover, HOVER_STEP)
    } else {
        tokens.field
    };

    let shape = RectShape::new(field, radius, fill, Stroke::NONE, StrokeKind::Inside);
    ui.painter().set(slot, shape);
    if focused {
        field_ring(ui, field, radius);
    }
}
