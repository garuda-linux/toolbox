//! The one look of every input, as Darkly's line edit draws it: a View
//! fill, a shadow under it, and a Highlight halo while it holds the focus.
//! At rest a 1 px card outline keeps the fill apart from a flat window. A
//! glyph before the text and a unit or a button after it are atoms of the
//! text edit, so the whole field is one hit area that focuses the text.

use egui::epaint::RectShape;
use egui::layers::ShapeIdx;
use egui::text_edit::TextEditState;
use egui::{
    Align2, Atom, Context, Frame, Id, IdSalt, Margin, Rect, Response, Role, Sense, Shadow, Shape,
    Stroke, StrokeKind, TextEdit, Ui, WidgetInfo, vec2,
};

use super::buttons::look::BLUR_PER_RADIUS;
use super::buttons::{SMALL_BUTTON, paint_small_button, push_height};
use super::focus::field_halo;
use super::rich;
use super::secret_field::revealed_id;
use crate::fonts::{BODY, Weight};
use crate::icons::Icon;
use crate::theme::FIELD_RADIUS;
use crate::tokens::{self, Tokens};
use crate::ui_ext::icon_image;

/// The text's padding on each side: Darkly's `LineEdit_FrameWidth` (5 px)
/// and the 2 px inset of the fill (darkly.h:56, darklystyle.cpp:3500-3509).
pub(super) const FRAME_WIDTH: i8 = 7;
/// Darkly's `Frame_FrameWidth`, the room between the widget and the fill
/// it paints (darkly.h:47, darklyhelper.cpp:1036).
pub(super) const INSET: f32 = 2.0;
/// The blur of the shadow under a field at rest (darklyhelper.cpp:1059),
/// doubled as egui's blur reads it.
const REST_BLUR: u8 = 5 * BLUR_PER_RADIUS;

/// What a field holds around its text. A unit and a button never go
/// together.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Parts<'a> {
    pub(super) glyph: Option<Icon>,
    pub(super) unit: Option<&'a str>,
    pub(super) button: Option<FieldButton<'a>>,
    /// No fill and no padding, for a control that paints the frame
    /// around the field and more.
    pub(super) inset: bool,
}

/// A small button at a field's end, such as Clear, named `name`.
#[derive(Clone, Copy, Debug)]
pub(super) struct FieldButton<'a> {
    pub(super) icon: Icon,
    pub(super) name: &'a str,
}

/// The field's height, the push button's, so a field and a button in one
/// row share their edges. Every single-line control is this tall.
pub(super) fn field_height(ui: &Ui) -> f32 {
    push_height(ui)
}

/// `edit` as a filled field [`field_height`] tall with
/// [`FIELD_RADIUS`](crate::theme::FIELD_RADIUS) corners. The text sits on
/// the field's middle and the caller keeps its width, hint and font.
pub fn text_field(ui: &mut Ui, edit: TextEdit<'_>) -> Response {
    field(ui, edit.align(Align2::LEFT_CENTER), Parts::default()).0
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
    let height = field_height(ui);
    let padding = if parts.inset { 0 } else { FRAME_WIDTH };

    // A button keeps as much room to the end as to the top.
    let end = if parts.button.is_some() {
        whole_points((height - SMALL_BUTTON) / 2.0)
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
        .min_size(vec2(0.0, height));

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
    // The theme's interact height would floor the field at the pill's 32 px,
    // the field is `height` tall on its own.
    let spacing = ui.spacing().icon_spacing;
    let interact = ui.spacing().interact_size.y;
    ui.spacing_mut().icon_spacing = 0.0;
    ui.spacing_mut().interact_size.y = 0.0;
    let output = edit.show(ui);
    ui.spacing_mut().icon_spacing = spacing;
    ui.spacing_mut().interact_size.y = interact;

    let button_rect = output.response.rect(button_salt);
    let response = output.response.response;
    let clicked = parts
        .button
        .zip(button_rect)
        .is_some_and(|(button, rect)| field_button(ui, response.id, rect, button));

    if !parts.inset {
        paint_field(ui, slot, response.rect, FIELD_RADIUS, response.has_focus());
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

/// Drawn after the text edit, so it lies over it and takes the click.
fn field_button(ui: &Ui, field: Id, rect: Rect, button: FieldButton<'_>) -> bool {
    let response = ui.interact(rect, field.with("button"), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), button.name));
    paint_small_button(ui, rect, &response, button.icon);
    response.on_hover_text(button.name).clicked()
}

/// `points` rounded up to whole points, as a margin holds them.
fn whole_points(points: f32) -> i8 {
    (0..=i8::MAX)
        .find(|&whole| f32::from(whole) >= points)
        .unwrap_or(i8::MAX)
}

/// A field's shadow, fill and focus halo in `slot` on the widget `field`
/// with `radius` corners, as Darkly's line edit paints them
/// (darklyhelper.cpp:1032-1066). The fill is inset two pixels from the
/// widget, and a focused one shrinks by one more under its halo.
pub(super) fn paint_field(ui: &Ui, slot: ShapeIdx, field: Rect, radius: u8, focused: bool) {
    let tokens = tokens::current(ui.ctx());
    let fill = field.shrink(INSET);
    let mut shapes = Vec::with_capacity(3);

    if focused && ui.is_enabled() {
        shapes.extend(field_halo(&tokens, fill, radius));
        shapes.push(filled(fill.shrink(1.0), radius.saturating_sub(1), &tokens));
    } else {
        if ui.is_enabled() {
            shapes.push(Shape::from(shadow(&tokens, fill, radius, REST_BLUR)));
        }

        shapes.push(rest(fill, radius, &tokens));
    }

    ui.painter().set(slot, Shape::Vec(shapes));
}

/// The input fill of a field, without a frame.
fn filled(rect: Rect, radius: u8, tokens: &Tokens) -> Shape {
    Shape::from(RectShape::filled(rect, radius, tokens.input))
}

/// The fill of a field at rest with the card outline around it, which
/// keeps it apart from a flat window. The outline is inside the fill.
fn rest(rect: Rect, radius: u8, tokens: &Tokens) -> Shape {
    Shape::from(RectShape::new(
        rect,
        radius,
        tokens.input,
        Stroke::new(1.0, tokens.card_outline),
        StrokeKind::Inside,
    ))
}

/// Darkly's box shadow under a field, one pixel down in the shadow shade
/// at 40 of 255 (darklyhelper.cpp:587-592).
fn shadow(tokens: &Tokens, rect: Rect, radius: u8, blur: u8) -> RectShape {
    Shadow {
        offset: [0, 1],
        blur,
        spread: 0,
        color: tokens.control_shadow,
    }
    .as_shape(rect, radius)
}
