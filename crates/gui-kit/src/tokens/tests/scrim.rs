//! The header's scrim over the live blur: neutral, muting and thick enough
//! that the header reads over the brightest page. And the dialog's scrim.

use egui::Theme;

use super::super::{Tokens, contrast};
use super::every_tokens;
use super::frost::laid_on;
use crate::palette::Palette;

/// The header's scrim is the window colour in a dark theme, as Breeze tints
/// its header with the window, so it stays near black.
#[test]
fn the_scrim_is_a_dark_window_tone_in_dark_and_the_window_in_light() {
    for (theme, tokens) in every_tokens() {
        let scrim = tokens.header_scrim;
        let rgb = [scrim.r(), scrim.g(), scrim.b()];

        match theme {
            Theme::Dark => {
                let brightest = rgb.iter().max().copied().unwrap_or(0);
                let spread = brightest - rgb.iter().min().copied().unwrap_or(0);
                assert!(brightest < 48, "{theme:?}: {scrim:?} is not dark");
                assert!(spread < 16, "{theme:?}: {scrim:?} tints too far");
            }
            Theme::Light => assert!(
                super::tinted_as(scrim, tokens.window),
                "{theme:?}: {scrim:?} is not the window colour"
            ),
        }
    }
}

#[test]
fn the_header_reads_on_the_band_scrim_alone_over_the_apps_own_surfaces() {
    for (theme, tokens) in every_tokens() {
        let own = [
            tokens.window,
            tokens.sidebar,
            tokens.card,
            tokens.field,
            tokens.button_hover,
        ];

        for backdrop in own {
            let laid = laid_on(tokens.header_scrim, backdrop);
            for (name, ink) in [("title", tokens.text), ("purpose", tokens.secondary)] {
                let ratio = contrast(ink, laid);
                assert!(
                    ratio >= 4.5,
                    "{theme:?} {name} over {backdrop:?}: {ratio:.2}"
                );
            }
        }
    }
}

#[test]
fn the_band_scrim_softens_an_accent_glow_and_lets_the_blur_show() {
    for (theme, tokens) in every_tokens() {
        let alpha = tokens.header_scrim.a();
        let glow = laid_on(tokens.header_scrim, tokens.accent);
        let page = laid_on(tokens.header_scrim, tokens.window);

        assert!(
            (80..=92).contains(&alpha),
            "{theme:?} {:?}: alpha {alpha}",
            tokens.accent
        );
        assert!(
            contrast(glow, page) < contrast(tokens.accent, tokens.window),
            "{theme:?} {:?}: the scrim does not soften the glow",
            tokens.accent
        );
    }
}

#[test]
fn the_dialog_scrim_is_the_shadow_shade_and_lighter_over_a_light_window() {
    let [dark, light] = [Theme::Dark, Theme::Light].map(|theme| {
        let tokens = Tokens::derive(&Palette::fallback(theme), theme);
        assert!(
            super::tinted_as(tokens.scrim, tokens.shadow_shade),
            "{theme:?}"
        );
        tokens.scrim.a()
    });

    assert!((110..=120).contains(&dark), "dark {dark}");
    assert!((60..=68).contains(&light), "light {light}");
}
