//! An app's picture, a file of any size and shape from the icon theme or
//! the `AppStream` cache. The theme worker reads and decodes it, so the
//! frame never waits for the disk or a large picture.

#[cfg(test)]
mod tests;

use std::io::Read as _;
use std::path::Path;

use egui::ColorImage;
use egui::load::SizeHint;
use image::imageops::FilterType;
use image::{ImageReader, Limits};
use resvg::usvg::Options;

/// What [`picture_uri`] puts before the path, so neither egui's file
/// loader nor the image loaders claim it.
pub(super) const PICTURE_URI: &str = "picture://";

/// Far above any app picture, which is a few kilobytes.
const MAX_BYTES: usize = 4 * 1024 * 1024;

/// A picture this wide decodes to 64 MiB, far above what a row draws.
const MAX_SIDE: u32 = 4096;

/// The URI to draw the picture at `path` with, through the theme worker.
pub fn picture_uri(path: &Path) -> String {
    format!("{PICTURE_URI}{}", path.display())
}

pub(super) fn path_of(uri: &str) -> Option<&Path> {
    uri.strip_prefix(PICTURE_URI).map(Path::new)
}

/// Reads at most [`MAX_BYTES`] and decodes at most [`MAX_SIDE`] a side,
/// then shrinks to `hint`, so a row keeps only what it draws.
pub(super) fn decode(path: &Path, hint: SizeHint, options: &Options) -> Result<ColorImage, String> {
    let bytes = read_capped(path)?;

    if path
        .extension()
        .is_some_and(|ending| ending.eq_ignore_ascii_case("svg"))
    {
        return egui_extras::image::load_svg_bytes_with_size(&bytes, hint, options);
    }

    let mut reader = ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|error| error.to_string())?;

    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    reader.limits(limits);

    let picture = reader.decode().map_err(|error| error.to_string())?;
    let (width, height) = fitted(picture.width(), picture.height(), hint);
    let shrunk = if (width, height) == (picture.width(), picture.height()) {
        picture
    } else {
        picture.resize(width, height, FilterType::Triangle)
    };

    let rgba = shrunk.to_rgba8();
    let size = [rgba.width(), rgba.height()].map(|side| usize::try_from(side).unwrap_or(0));
    Ok(ColorImage::from_rgba_unmultiplied(size, rgba.as_raw()))
}

fn read_capped(path: &Path) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let limit = u64::try_from(MAX_BYTES).unwrap_or(u64::MAX);
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;

    if bytes.len() > MAX_BYTES {
        return Err(format!("larger than {MAX_BYTES} bytes"));
    }

    Ok(bytes)
}

/// The largest size within `hint` that keeps the picture's shape and
/// never grows it.
fn fitted(width: u32, height: u32, hint: SizeHint) -> (u32, u32) {
    let (most_wide, most_high) = match hint {
        SizeHint::Size {
            width: most_wide,
            height: most_high,
            maintain_aspect_ratio: _,
        } => (most_wide, most_high),
        SizeHint::Width(most) => (most, u32::MAX),
        SizeHint::Height(most) => (u32::MAX, most),
        SizeHint::Scale(_) => (width, height),
    };

    if width <= most_wide && height <= most_high {
        return (width, height);
    }

    let scale =
        (f64::from(most_wide) / f64::from(width)).min(f64::from(most_high) / f64::from(height));

    (side(width, scale), side(height, scale))
}

#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the scale is below one, so the side stays within u32"
)]
fn side(pixels: u32, scale: f64) -> u32 {
    ((f64::from(pixels) * scale).round() as u32).max(1)
}
