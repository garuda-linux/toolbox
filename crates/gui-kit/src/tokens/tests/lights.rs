use egui::Theme;

use super::super::contrast;
use super::{every_tokens, lightness};

#[test]
fn a_dark_cards_top_light_is_a_subtle_step_above_the_card() {
    for (theme, tokens) in every_tokens() {
        if theme != Theme::Dark {
            continue;
        }

        let ratio = contrast(tokens.top_light, tokens.card);
        assert!(
            lightness(tokens.top_light) > lightness(tokens.card),
            "{tokens:?}"
        );
        assert!(ratio > 1.1 && ratio < 1.6, "{ratio:.2}:1");
    }
}

#[test]
fn a_skeletons_band_stands_apart_from_its_bars_lighter_in_dark_and_darker_in_light() {
    // The sweep moves toward the light end in a dark theme and toward the
    // shadow shade in a light one, by the same margin.
    for (theme, tokens) in every_tokens() {
        let ratio = contrast(tokens.shimmer, tokens.button_hover);
        match theme {
            Theme::Dark => assert!(
                lightness(tokens.shimmer) > lightness(tokens.button_hover),
                "{theme:?}"
            ),
            Theme::Light => assert!(
                lightness(tokens.shimmer) < lightness(tokens.button_hover),
                "{theme:?}"
            ),
        }

        assert!(ratio > 1.05 && ratio < 1.5, "{theme:?}: {ratio:.2}:1");
    }
}
