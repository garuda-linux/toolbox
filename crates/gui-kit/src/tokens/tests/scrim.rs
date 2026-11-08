//! The header's scrim over the live blur: neutral, muting and thick enough
//! that the header reads over the brightest page.

use egui::Theme;

use super::super::contrast;
use super::{every_tokens, laid_on};

#[test]
fn the_scrim_is_neutral_black_in_dark_and_white_in_light() {
    for (theme, tokens) in every_tokens() {
        let scrim = tokens.header_scrim;
        let shade = match theme {
            Theme::Dark => 0,
            Theme::Light => scrim.a(),
        };

        assert_eq!(
            [scrim.r(), scrim.g(), scrim.b()],
            [shade; 3],
            "{theme:?}: {scrim:?} casts a tint"
        );
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
            tokens.hover,
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
            (46..=56).contains(&alpha),
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
