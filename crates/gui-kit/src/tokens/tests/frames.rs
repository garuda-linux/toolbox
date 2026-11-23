use egui::Theme;

use super::super::darkly::darker;
use super::super::mix;
use crate::palette::Palette;
use crate::tokens::Tokens;

fn each_theme(check: impl Fn(Theme, &Palette, &Tokens)) {
    for theme in [Theme::Dark, Theme::Light] {
        let palette = Palette::fallback(theme);
        let tokens = Tokens::derive(&palette, theme);
        check(theme, &palette, &tokens);
    }
}

/// A card is Darkly's group box fill, the window mixed toward the View by
/// 0.3 (darklyhelper.cpp:180), in both themes.
#[test]
fn a_card_is_the_group_box_fill_in_both_themes() {
    each_theme(|theme, palette, tokens| {
        let frame = mix(palette.window, palette.view, 0.3);
        let expected = if crate::tokens::contrast(palette.view, palette.window) < 1.1 {
            mix(frame, palette.text, 0.04)
        } else {
            frame
        };

        assert_eq!(tokens.card, expected, "{theme:?}");
    });
}

/// A dark View gets `Qt.darker(View, 1.2)` as its border, a light one the
/// View mixed toward the text by 0.15 (`FormCardBackground.qml`). The border
/// is of the View, not of the Darkly card, which has none.
#[test]
fn a_card_border_follows_the_form_card_rule_in_both_themes() {
    each_theme(|theme, palette, tokens| {
        let expected = if theme == Theme::Dark {
            darker(palette.view, 1.2)
        } else {
            mix(palette.view, palette.text, 0.15)
        };

        assert_eq!(tokens.outline, expected, "{theme:?}");
    });
}

/// The window is the desktop's Window colour, as Breeze paints it, so no
/// lightened window stands in for it.
#[test]
fn the_window_is_the_desktops_window_in_both_themes() {
    each_theme(|theme, palette, tokens| {
        assert_eq!(tokens.window, palette.window, "{theme:?}");
    });
}
