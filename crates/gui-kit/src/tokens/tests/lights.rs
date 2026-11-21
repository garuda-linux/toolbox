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
fn a_skeletons_band_is_lighter_than_its_bars() {
    for (theme, tokens) in every_tokens() {
        let ratio = contrast(tokens.shimmer, tokens.hover);
        assert!(
            lightness(tokens.shimmer) > lightness(tokens.hover),
            "{theme:?}"
        );
        assert!(ratio > 1.05 && ratio < 1.5, "{theme:?}: {ratio:.2}:1");
    }
}
