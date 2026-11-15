//! The guide's picture files. The dark capture is `name.png`, the light one
//! `name-light.png`, both as RGB, which a frame is opaque in.

use std::fs;
use std::path::{Path, PathBuf};

use egui::Theme;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{DynamicImage, ExtendedColorType, ImageEncoder as _, RgbImage, RgbaImage};

/// Writes the renders instead of checking the committed files when set to 1.
const UPDATE: &str = "UPDATE_GUIDE_IMAGES";

pub fn updating() -> bool {
    std::env::var(UPDATE).is_ok_and(|value| value == "1")
}

/// The folder of the guide's pictures, beside the pages they illustrate.
fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/src/images")
}

fn path(name: &str, theme: Theme) -> PathBuf {
    match theme {
        Theme::Dark => folder().join(format!("{name}.png")),
        Theme::Light => folder().join(format!("{name}-light.png")),
    }
}

/// Writes `picture` as the file of `name` in `theme` when `update`, else
/// reports the file when it is missing or differs from `picture`.
pub fn settle(name: &str, theme: Theme, picture: &RgbaImage, update: bool) -> Result<(), String> {
    let path = path(name, theme);
    let rgb = DynamicImage::ImageRgba8(picture.clone()).into_rgb8();
    if update {
        return fs::write(&path, encode(&rgb)).map_err(|err| format!("{}: {err}", path.display()));
    }

    let committed = fs::read(&path).map_err(|err| format!("{}: {err}", path.display()))?;
    let decoded = image::load_from_memory(&committed)
        .map_err(|err| format!("{}: {err}", path.display()))?
        .into_rgb8();

    if same_picture(&decoded, &rgb) {
        Ok(())
    } else {
        let fresh = path.with_extension("new.png");
        fs::write(&fresh, encode(&rgb)).map_err(|err| format!("{}: {err}", fresh.display()))?;
        Err(format!(
            "{} differs from a fresh render, see {}",
            path.display(),
            fresh.display()
        ))
    }
}

fn encode(rgb: &RgbImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    PngEncoder::new_with_quality(&mut bytes, CompressionType::Best, FilterType::Adaptive)
        .write_image(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            ExtendedColorType::Rgb8,
        )
        .expect("a PNG encodes");
    bytes
}

/// Equal within the tolerance of the kittest snapshots (kittest.toml), as
/// the CPU renderer's anti-aliasing moves a few edge pixels between runs.
fn same_picture(committed: &RgbImage, fresh: &RgbImage) -> bool {
    const THRESHOLD: f32 = 1.0;
    const MAX_FAILED_PIXELS: i32 = 10;

    if committed.dimensions() != fresh.dimensions() {
        return false;
    }

    let rgba = |picture: &RgbImage| DynamicImage::ImageRgb8(picture.clone()).into_rgba8();
    dify::diff::get_results(
        rgba(committed),
        rgba(fresh),
        THRESHOLD,
        true,
        None,
        &None,
        &None,
    )
    .is_none_or(|(failed, _)| failed <= MAX_FAILED_PIXELS)
}
