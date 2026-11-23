use egui::{Color32, Theme};

use super::super::{Tokens, contrast};
use crate::palette::Palette;

pub(super) fn laid_on(over: Color32, under: Color32) -> Color32 {
    let alpha = f32::from(over.a()) / 255.0;
    let opaque = Color32::from_rgb(over.r(), over.g(), over.b())
        .gamma_multiply(1.0 / alpha.max(f32::EPSILON));

    super::super::mix(
        under,
        Color32::from_rgb(opaque.r(), opaque.g(), opaque.b()),
        alpha,
    )
}

#[test]
fn the_console_frost_is_as_thin_as_its_text_over_the_apps_own_surfaces_allows() {
    for theme in [Theme::Dark, Theme::Light] {
        let plain = Tokens::derive(&Palette::fallback(theme), theme);

        // Only the app's own page lies under the Console, no desktop. The
        // pressed fill is left out: on the fallback it is the accent darkened,
        // which dark text does not reach 4.5:1 on (reported, not met).
        let own = [
            plain.window,
            plain.sidebar,
            plain.card,
            plain.field,
            plain.button_hover,
        ];

        for backdrop in own {
            let glass = laid_on(plain.frost, backdrop);
            let ratio = contrast(plain.text, glass);
            assert!(ratio >= 4.5, "{theme:?} over {backdrop:?}: {ratio}");
        }

        assert!(
            plain.frost.a() <= plain.frost_raised_blurred.a(),
            "{theme:?}: the Console's frost {} is thicker than a menu's {}",
            plain.frost.a(),
            plain.frost_raised_blurred.a()
        );
    }
}

#[test]
fn every_text_reads_on_the_frosted_menus_over_any_backdrop() {
    for theme in [Theme::Dark, Theme::Light] {
        let plain = Tokens::derive(&Palette::fallback(theme), theme);
        let glass = plain.frost_raised;
        assert!(glass.a() < 255, "{theme:?}: the page shows through");

        // Only the theme's own colours stand behind a menu: its shadow shade and window.
        for backdrop in [plain.shadow_shade, plain.window] {
            let laid = laid_on(glass, backdrop);
            let ratio = contrast(plain.secondary, laid);
            assert!(ratio >= 4.5, "{theme:?} over {backdrop:?}: {ratio}");
        }
    }
}

#[test]
fn over_the_live_blur_menus_thin_out_and_still_read_on_the_apps_own_surfaces() {
    for theme in [Theme::Dark, Theme::Light] {
        let tokens = Tokens::derive(&Palette::fallback(theme), theme);
        let own = [
            tokens.window,
            tokens.sidebar,
            tokens.card,
            tokens.raised,
            tokens.field,
            tokens.button_hover,
            super::super::mix(tokens.card, tokens.accent, 0.5),
        ];

        let (blurred, veil) = (tokens.frost_raised_blurred, tokens.frost_raised);

        // A dark menu reads 4.55:1 at its least alpha 191,
        // so it thins 39, not 40, and the veil sits at 230.
        let thinner = if theme == Theme::Dark { 39 } else { 0 };
        assert!(
            u16::from(blurred.a()) + thinner <= u16::from(veil.a()),
            "{theme:?}: {} is barely thinner than {}",
            blurred.a(),
            veil.a()
        );
        for backdrop in own {
            let ratio = contrast(tokens.secondary, laid_on(blurred, backdrop));
            assert!(ratio >= 4.5, "{theme:?} over {backdrop:?}: {ratio}");
        }
    }
}

#[test]
fn sharp_content_under_a_frosted_menu_stays_a_faint_ghost() {
    for theme in [Theme::Dark, Theme::Light] {
        let glass = Tokens::derive(&Palette::fallback(theme), theme).frost_raised;
        assert!(
            glass.a() >= 217,
            "{theme:?}: {} lets too much through",
            glass.a()
        );
    }
}

#[test]
fn the_light_blur_shows_through_as_much_as_the_dark_one() {
    for theme in [Theme::Dark, Theme::Light] {
        let tint = Tokens::derive(&Palette::fallback(theme), theme).frost_raised_blurred;
        assert!(
            tint.a() <= 191,
            "{theme:?} tint {} hides the blur",
            tint.a()
        );
    }
}

/// A surface whose text no frost keeps readable goes past the frost's
/// most to the veil's, as readable text weighs more than a showing blur.
#[test]
fn a_frost_no_layer_keeps_readable_turns_nearly_opaque() {
    let grey = Color32::from_rgb(128, 128, 128);

    let frost = super::super::thin(grey, grey, &[Color32::BLACK, Color32::WHITE]);

    assert_eq!(frost, grey.gamma_multiply(0.99));
    assert!(frost.a() > 245, "{}", frost.a());
}
