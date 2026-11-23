use egui::Theme;

use super::super::{Tokens, contrast, mix};
use crate::desktop::kde::parse;
use crate::palette::Palette;
use crate::testing::desktop::kde::{BREEZEDARK_COLORS, INSTALLED_KDEGLOBALS};

/// The Dark fallback with the colours of a real desktop capture laid over it.
pub(super) fn captured(text: &str) -> Palette {
    parse(text).colors.over(Palette::fallback(Theme::Dark))
}

/// The Mokka capture is Catppuccin Mocha, whose View is the Window colour.
#[test]
fn a_card_steps_toward_the_text_where_view_is_the_window_colour() {
    let palette = captured(INSTALLED_KDEGLOBALS);
    let tokens = Tokens::derive(&palette, Theme::Dark);

    assert!(contrast(palette.view, palette.window) < 1.1);
    assert_eq!(
        tokens.card,
        mix(mix(palette.window, palette.view, 0.3), tokens.text, 0.04)
    );
    assert!(contrast(tokens.card, tokens.window) > 1.05);
}

/// Breeze Dark's View stands apart from its Window, so the card is Darkly's
/// group box fill as it is.
#[test]
fn a_card_is_the_group_box_fill_where_view_stands_apart_from_the_window() {
    let palette = captured(BREEZEDARK_COLORS);
    let tokens = Tokens::derive(&palette, Theme::Dark);

    assert!(contrast(palette.view, palette.window) >= 1.1);
    assert_eq!(tokens.card, mix(palette.window, palette.view, 0.3));
}

/// Darkly's frame outline (darklyhelper.cpp:127): the window mixed toward the
/// text by a quarter, in both captures.
#[test]
fn the_card_outline_is_the_window_mixed_toward_the_text_by_a_quarter() {
    for capture in [INSTALLED_KDEGLOBALS, BREEZEDARK_COLORS] {
        let palette = captured(capture);
        let tokens = Tokens::derive(&palette, Theme::Dark);

        assert_eq!(tokens.card_outline, mix(tokens.window, tokens.text, 0.25));
    }
}

/// The three palettes a field and a tab are checked on: the dark and the
/// light fallback, and the Mokka capture laid over the dark one.
fn flat_palettes() -> [(&'static str, Theme, Palette); 3] {
    [
        ("dark fallback", Theme::Dark, Palette::fallback(Theme::Dark)),
        (
            "light fallback",
            Theme::Light,
            Palette::fallback(Theme::Light),
        ),
        ("Mokka", Theme::Dark, captured(INSTALLED_KDEGLOBALS)),
    ]
}

/// A field fill and its card outline both read apart from the window, on
/// every palette, as a card does. The numbers are the contrast ratios.
#[test]
fn a_field_stands_apart_from_the_window_on_flat_palettes() {
    for (name, theme, palette) in flat_palettes() {
        let tokens = Tokens::derive(&palette, theme);
        let fill = contrast(tokens.input, tokens.window);
        let outline = contrast(tokens.card_outline, tokens.window);

        assert!(fill > 1.0, "{name}: input fill {fill:.2}:1 to the window");
        assert!(
            outline >= 1.1,
            "{name}: card outline {outline:.2}:1 to the window"
        );
    }
}

/// The edge of a frameless control is its card outline, which must read
/// apart from the window and the card, as the fill alone is too faint on
/// flat palettes.
#[test]
fn a_control_edge_reads_apart_from_the_window_and_the_card_on_flat_palettes() {
    for (name, theme, palette) in flat_palettes() {
        let tokens = Tokens::derive(&palette, theme);
        for (label, surface) in [("window", tokens.window), ("card", tokens.card)] {
            let outline = contrast(tokens.card_outline, surface);
            assert!(
                outline >= 1.1,
                "{name}: card outline {outline:.2}:1 to the {label}"
            );
        }
    }
}

/// The selected tab is the push-button colour, which stands apart from the
/// window on every palette.
#[test]
fn a_chosen_tab_stands_apart_from_the_window_on_flat_palettes() {
    for (name, theme, palette) in flat_palettes() {
        let tokens = Tokens::derive(&palette, theme);
        let pill = contrast(tokens.input, tokens.window);

        assert!(pill > 1.0, "{name}: chosen tab {pill:.2}:1 to the window");
    }
}
