//! The guide's pictures. A 1 px outline and a soft shadow set each one off
//! the page, so a dark capture still has an edge in the light theme. A
//! click shows it as large as the window allows, where a click, Escape or
//! Close lead back.

use egui::accesskit::Role;
use egui::{CornerRadius, CursorIcon, Frame, Id, Response, Sense, Shadow, Stroke, Ui, Vec2, vec2};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::theme::CARD_RADIUS;
use toolbox_gui_kit::widgets::{DialogWidth, ModalWrapper, focus_ring, rich};
use toolbox_guide::Image;

use super::markdown::Reader;
use crate::i18n;

const OUTLINE: f32 = 1.0;
const SHADOW: Shadow = Shadow {
    offset: [0, 2],
    blur: 12,
    spread: 0,
    color: egui::Color32::TRANSPARENT,
};

/// A picture of the guide with its description under it, or the
/// description alone when the guide lacks the picture.
pub(super) fn show(ui: &mut Ui, image: &Image, reader: &mut Reader<'_>) {
    let tokens = ui.tokens();
    if let Some(bytes) = toolbox_guide::image(&image.src) {
        let width = OUTLINE.mul_add(-2.0, ui.available_width()).max(0.0);
        let size = shown_size(bytes, vec2(width, f32::INFINITY));
        let name = i18n::text_with(ui.ctx(), "guide-enlarge", &[("alt", &image.alt)]);
        let response = framed(ui, &image.src, bytes, size, &image.alt);
        response.widget_info(|| egui::WidgetInfo::labeled(Role::Button, true, &*name));

        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::ZoomIn);
        }

        if response.clicked() {
            reader.enlarged = Some((image.src.clone(), image.alt.clone()));
        }
    }

    ui.label(rich(
        ui,
        &image.alt,
        Weight::Regular,
        SMALL,
        tokens.secondary,
    ));
}

/// The picture `src` over the page, until it is closed. Returns whether it
/// is still open.
pub(super) fn lightbox(ctx: &egui::Context, src: &str, alt: &str) -> bool {
    let Some(bytes) = toolbox_guide::image(src) else {
        return false;
    };

    let close = i18n::text(ctx, "dialog-close");
    let outcome = ModalWrapper::new(Id::unique("toolbox_gui::guide::lightbox"), alt)
        .title_bar(&close)
        .width(DialogWidth::Window)
        .full_height()
        .show(ctx, |ui| {
            let room = ui.available_size();
            let size = shown_size(bytes, room);
            ui.vertical_centered(|ui| framed(ui, src, bytes, size, alt).clicked())
                .inner
        });

    !(outcome.closed || outcome.inner)
}

/// The picture at `size`, its whole room kept from the first frame, in its
/// outline and shadow. The picture is decoded on a thread of its own and no
/// spinner shows meanwhile, so the words below it never move.
fn framed(ui: &mut Ui, src: &str, bytes: &'static [u8], size: Option<Vec2>, alt: &str) -> Response {
    let tokens = ui.tokens();
    let uri = format!("bytes://guide/{src}");
    let picture = egui::Image::from_bytes(uri, bytes)
        .corner_radius(CornerRadius::same(CARD_RADIUS))
        .alt_text(alt)
        .sense(Sense::click())
        .show_loading_spinner(false);

    let shadow = Shadow {
        color: tokens.shadow,
        ..SHADOW
    };

    let response = Frame::new()
        .stroke(Stroke::new(OUTLINE, tokens.outline))
        .shadow(shadow)
        .corner_radius(CornerRadius::same(CARD_RADIUS))
        .show(ui, |ui| match size {
            Some(size) => ui.add_sized(size, picture.fit_to_exact_size(size)),
            None => ui.add(picture.fit_to_exact_size(vec2(ui.available_width(), f32::INFINITY))),
        })
        .inner;

    focus_ring(ui, &response, CARD_RADIUS);
    response
}

/// The picture's size scaled to fit `room`, never above its own.
fn shown_size(bytes: &[u8], room: Vec2) -> Option<Vec2> {
    let (picture_width, picture_height) = toolbox_guide::image_size(bytes)?;
    let picture_width = f32::from(u16::try_from(picture_width).ok()?);
    let picture_height = f32::from(u16::try_from(picture_height).ok()?);
    if picture_width <= 0.0 || picture_height <= 0.0 {
        return None;
    }

    let scale = (room.x / picture_width).min(room.y / picture_height);
    Some(vec2(picture_width, picture_height) * scale.min(1.0))
}
