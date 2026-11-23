#![expect(
    clippy::disallowed_methods,
    reason = "tests compare against literal colours"
)]

use egui::{Color32, Theme};

use super::Palette;

#[test]
fn fallback_dark_is_catppuccin_mocha_with_mauve() {
    let palette = Palette::fallback(Theme::Dark);
    assert_eq!(palette.window, Color32::from_rgb(0x1e, 0x1e, 0x2e));
    assert_eq!(palette.text, Color32::from_rgb(0xcd, 0xd6, 0xf4));
    assert_eq!(palette.accent, Color32::from_rgb(0xcb, 0xa6, 0xf7));
}

#[test]
fn fallback_light_is_catppuccin_latte_with_mauve() {
    let palette = Palette::fallback(Theme::Light);
    assert_eq!(palette.window, Color32::from_rgb(0xef, 0xf1, 0xf5));
    assert_eq!(palette.text, Color32::from_rgb(0x4c, 0x4f, 0x69));
    assert_eq!(palette.accent, Color32::from_rgb(0x88, 0x39, 0xef));
}

#[test]
fn the_palette_in_use_is_the_fallback_before_any_was_stored() {
    let ctx = egui::Context::default();
    ctx.set_theme(Theme::Dark);
    assert_eq!(super::current(&ctx), Palette::fallback(Theme::Dark));
}

#[test]
fn the_fallbacks_name_none_for_darklys_roles_so_the_tokens_derive_them() {
    // Darkly's hover, focus and tooltips take the accent, the raised surface
    // and the text, which the built-in palettes already hold. No literal
    // colour is added for them.
    for palette in [
        Palette::fallback(Theme::Dark),
        Palette::fallback(Theme::Light),
        Palette::adwaita(Theme::Dark),
        Palette::adwaita(Theme::Light),
    ] {
        assert_eq!(palette.decoration_hover, None);
        assert_eq!(palette.decoration_focus, None);
        assert_eq!(palette.tooltip, None);
        assert_eq!(palette.tooltip_text, None);
        assert_eq!(palette.contrast, None);
    }
}
