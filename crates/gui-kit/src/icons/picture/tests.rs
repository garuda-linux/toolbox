use std::io::Cursor;
use std::path::{Path, PathBuf};

use egui::load::SizeHint;
use image::{ImageFormat, RgbaImage};

use super::{MAX_BYTES, MAX_SIDE, decode, path_of, picture_uri};
use crate::icons::svg::options;

const SQUARE: SizeHint = SizeHint::Size {
    width: 48,
    height: 48,
    maintain_aspect_ratio: false,
};

/// A file of its own below the test's temporary folder, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn with(name: &str, bytes: &[u8]) -> Self {
        let path = std::env::temp_dir().join(format!(
            "toolbox-gui-kit-picture-{}-{name}",
            std::process::id()
        ));

        std::fs::write(&path, bytes).unwrap();
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0).ok();
    }
}

fn png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    RgbaImage::from_pixel(width, height, image::Rgba([0xe0, 0x40, 0x80, 0xff]))
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .unwrap();
    bytes
}

#[test]
fn a_large_picture_shrinks_to_the_drawn_size() {
    let file = Scratch::with("large.png", &png(512, 256));

    let picture = decode(&file.0, SQUARE, &options()).unwrap();

    assert_eq!(picture.size, [48, 24]);
}

#[test]
fn a_small_picture_keeps_its_size() {
    let file = Scratch::with("small.png", &png(16, 16));

    assert_eq!(decode(&file.0, SQUARE, &options()).unwrap().size, [16, 16]);
}

#[test]
fn a_file_above_the_byte_cap_is_not_decoded() {
    let mut bytes = png(4, 4);
    bytes.resize(MAX_BYTES + 1, 0);
    let file = Scratch::with("huge.png", &bytes);

    decode(&file.0, SQUARE, &options()).unwrap_err();
}

/// A few hundred bytes on disk, tens of megabytes once decoded.
#[test]
fn a_picture_above_the_side_limit_is_not_decoded() {
    let file = Scratch::with("wide.png", &png(MAX_SIDE + 1, 1));

    decode(&file.0, SQUARE, &options()).unwrap_err();
}

#[test]
fn an_svg_renders_at_the_drawn_size() {
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512"><rect width="512" height="512" fill="#e04080"/></svg>"##;
    let file = Scratch::with("app.svg", svg);

    assert_eq!(decode(&file.0, SQUARE, &options()).unwrap().size, [48, 48]);
}

#[test]
fn the_uri_names_the_file() {
    let path = Path::new("/usr/share/icons/hicolor/48x48/apps/firefox.png");
    let uri = picture_uri(path);

    assert_eq!(path_of(&uri), Some(path));
    assert_eq!(path_of("file:///usr/share/icons/x.png"), None);
}
