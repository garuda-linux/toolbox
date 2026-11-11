//! A package's picture in a row: the icon theme's or the `AppStream`
//! one, the initial of its name on a letter tile while there is none.

#[cfg(test)]
mod tests;

use egui::emath::GuiRounding as _;
use egui::load::{SizeHint, TexturePoll};
use egui::{Image, Rect, TextureOptions, Ui};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{BODY, Weight, font};
use toolbox_gui_kit::widgets::{paint_letter_tile, title_capitals_top};
use toolbox_plan::PackageName;

use super::{PICTURE_RADIUS, PICTURE_SIZE};
use crate::pictures::Pictures;

/// The hue comes from `package`, so no language changes the colour.
#[derive(Clone, Copy, Debug)]
pub struct Initial<'name> {
    pub shown: &'name str,
    pub package: &'name str,
}

impl<'name> Initial<'name> {
    pub fn named(package: &'name PackageName) -> Self {
        Self {
            shown: package.as_str(),
            package: package.as_str(),
        }
    }
}

/// Paints the picture of `package` into `at`. The first call asks for
/// it, so a row shows the letter tile until the picture was found.
pub fn package_picture(
    ui: &Ui,
    pictures: Option<&Pictures>,
    package: Option<&PackageName>,
    initial: Initial<'_>,
    at: Rect,
) {
    let found = pictures
        .zip(package)
        .and_then(|(finder, name)| finder.file(name).map(|file| (finder, file)));

    let Some((finder, file)) = found else {
        paint_letter_tile(ui, at, PICTURE_RADIUS, initial.shown, initial.package);
        return;
    };

    let uri = format!("file://{}", file.display());
    if picture_or_initial(ui, &uri, at, initial) {
        finder.drawn(ui.ctx(), &uri);
    }
}

/// Paints the picture at `uri` into `at` once it is loaded, the letter
/// tile while it loads or when it cannot be read. Returns whether the
/// picture was painted.
pub fn picture_or_initial(ui: &Ui, uri: &str, at: Rect, initial: Initial<'_>) -> bool {
    // Trace: egui rasterizes an SVG in the frame that first draws it.
    let _picture = tracing::trace_span!("row picture").entered();

    // The drawn size, so a vector picture is drawn at it rather than at
    // its own, often far larger, size.
    let image = Image::new(uri)
        .fit_to_exact_size(at.size())
        .corner_radius(PICTURE_RADIUS)
        .show_loading_spinner(false);

    if loaded(ui, &image, at) {
        image.paint_at(ui, at);
        return true;
    }

    paint_letter_tile(ui, at, PICTURE_RADIUS, initial.shown, initial.package);
    false
}

/// Whether `image` is ready at the size [`Image::paint_at`] asks the
/// loaders for, so both share one texture.
fn loaded(ui: &Ui, image: &Image<'_>, at: Rect) -> bool {
    let rect = at.round_to_pixels(ui.pixels_per_point());
    let pixels = (ui.pixels_per_point() * rect.size()).round();
    let hint = SizeHint::Size {
        width: pixel_count(pixels.x),
        height: pixel_count(pixels.y),
        maintain_aspect_ratio: false,
    };

    matches!(
        image
            .source(ui.ctx())
            .load(ui.ctx(), TextureOptions::default(), hint),
        Ok(TexturePoll::Ready { texture: _ })
    )
}

/// The pixels of a rounded edge, as egui's own `paint_at` counts them.
#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a row's picture is a few dozen whole pixels wide"
)]
const fn pixel_count(pixels: f32) -> u32 {
    pixels as u32
}

/// How far below the row's top the picture starts: at the top of the
/// title's capitals, so the title never looks pushed down beside it.
/// `wrap` as the row's [`title_line`](toolbox_gui_kit::widgets::title_line).
pub fn picture_top(ui: &Ui, wrap: bool) -> f32 {
    title_capitals_top(ui, font(ui.style(), Weight::SemiBold, BODY), wrap).max(0.0)
}

pub fn picture_height(ui: &Ui, wrap: bool) -> f32 {
    picture_top(ui, wrap) + PICTURE_SIZE
}

/// Returns the square, its top on the top of the title's capitals
/// whatever follows the title.
///
/// egui's item spacing after a widget would widen the gap, so it is off
/// for the square alone. The text keeps it.
pub fn picture_slot(ui: &mut Ui, wrap: bool) -> Rect {
    let top = picture_top(ui, wrap);
    let spacing = ui.spacing().item_spacing.x;
    ui.spacing_mut().item_spacing.x = 0.0;
    let room = egui::vec2(PICTURE_SIZE, top + PICTURE_SIZE);
    let (slot, _) = ui.allocate_exact_size(room, egui::Sense::hover());
    ui.spacing_mut().item_spacing.x = spacing;
    ui.add_space(ui.tokens().sizes.picture_gap);

    Rect::from_min_size(
        egui::pos2(slot.left(), slot.top() + top),
        egui::Vec2::splat(PICTURE_SIZE),
    )
}
