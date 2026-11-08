#![expect(
    clippy::disallowed_methods,
    reason = "tests compare against literal colours"
)]

use egui::{Color32, Theme};

use super::Colors;
use crate::Palette;

#[test]
fn no_colours_leave_the_base_untouched() {
    let base = Palette::fallback(Theme::Dark);
    assert_eq!(Colors::default().over(base), base);
}

#[test]
fn set_roles_replace_the_base() {
    let base = Palette::fallback(Theme::Dark);
    let accent = Color32::from_rgb(0x35, 0x84, 0xe4);
    let palette = Colors {
        accent: Some(accent),
        ..Colors::default()
    }
    .over(base);

    assert_eq!(palette.accent, accent);
    assert_eq!(palette.window, base.window);
}

#[test]
fn link_colours_reach_the_palette_only_when_set() {
    let base = Palette::fallback(Theme::Dark);
    let link = Color32::from_rgb(0x1d, 0x99, 0xf3);
    let palette = Colors {
        link: Some(link),
        ..Colors::default()
    }
    .over(base);

    assert_eq!(palette.link, Some(link));
    assert_eq!(palette.visited, None);
}
