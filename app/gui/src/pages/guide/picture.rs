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
use crate::i18n::Arg;

const OUTLINE: f32 = 1.0;
const PLACEHOLDER_HEIGHT: f32 = 120.0;
const SHADOW: Shadow = Shadow {
    offset: [0, 2],
    blur: 12,
    spread: 0,
    color: egui::Color32::TRANSPARENT,
};

/// A picture of the guide with its description under it, or a placeholder
/// when the guide lacks the picture.
pub(super) fn show(ui: &mut Ui, image: &Image, reader: &mut Reader<'_>) {
    let tokens = ui.tokens();
    let dark = ui.ctx().theme() == egui::Theme::Dark;
    let shown = toolbox_guide::image_for(&image.src, dark);
    if let Some(bytes) = shown {
        let width = OUTLINE.mul_add(-2.0, ui.available_width()).max(0.0);
        let size = shown_size(bytes, vec2(width, f32::INFINITY));
        let name = i18n::text_with(ui.ctx(), "guide-enlarge", &[("alt", Arg::from(&image.alt))]);
        let response = framed(ui, picture_key(&image.src, dark), bytes, size, &image.alt);
        response.widget_info(|| egui::WidgetInfo::labeled(Role::Button, true, &*name));

        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::ZoomIn);
        }

        if response.clicked() {
            reader.enlarged = Some((image.src.clone(), image.alt.clone()));
        }
    } else {
        placeholder(ui);
    }

    let caption = ui.label(rich(
        ui,
        &image.alt,
        Weight::Regular,
        SMALL,
        tokens.secondary,
    ));

    // The picture's button already says the description, so a screen
    // reader would hear it twice.
    if shown.is_some() {
        ui.ctx()
            .accesskit_node_builder(caption.id, egui::accesskit::Node::set_hidden);
    }
}

/// Stands where a picture the guide lacks would be, so the description
/// under it still reads as a caption.
fn placeholder(ui: &mut Ui) {
    let tokens = ui.tokens();
    let text = i18n::text(ui.ctx(), "guide-picture-missing");
    let size = vec2(ui.available_width(), PLACEHOLDER_HEIGHT);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let corners = CornerRadius::same(CARD_RADIUS);
    ui.painter().rect(
        rect,
        corners,
        tokens.field,
        Stroke::new(OUTLINE, tokens.outline),
        egui::StrokeKind::Inside,
    );

    let words = rich(ui, &text, Weight::Regular, SMALL, tokens.secondary);
    ui.put(rect, egui::Label::new(words));
}

/// The picture `src` over the page, until it is closed. Returns whether it
/// is still open.
pub(super) fn lightbox(ctx: &egui::Context, src: &str, alt: &str) -> bool {
    let dark = ctx.theme() == egui::Theme::Dark;
    let Some(bytes) = toolbox_guide::image_for(src, dark) else {
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
            ui.vertical_centered(|ui| {
                framed(ui, picture_key(src, dark), bytes, size, alt).clicked()
            })
            .inner
        });

    !(outcome.closed || outcome.inner)
}

/// The image cache key of a picture in a theme. Each theme's capture has its
/// own key, so the cache keeps both.
fn picture_key(src: &str, dark: bool) -> String {
    let theme = if dark { "dark" } else { "light" };
    format!("bytes://guide/{theme}/{src}")
}

/// The picture at `size`, its whole room kept from the first frame, in its
/// outline and shadow. The picture is decoded on a thread of its own and no
/// spinner shows meanwhile, so the words below it never move.
fn framed(
    ui: &mut Ui,
    uri: String,
    bytes: &'static [u8],
    size: Option<Vec2>,
    alt: &str,
) -> Response {
    let tokens = ui.tokens();
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
