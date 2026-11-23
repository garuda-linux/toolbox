use egui::Theme;

use super::super::{Tokens, contrast, darkly};
use crate::desktop::kde::parse;
use crate::palette::Palette;
use crate::testing::desktop::kde::{BREEZEDARK_COLORS, INSTALLED_KDEGLOBALS};

/// The focus fill a push button takes under the pointer, as `push_fill` draws it.
fn focus_under_pointer(decoration_focus: egui::Color32) -> egui::Color32 {
    darkly::lighter(darkly::lighter(decoration_focus, 1.15), 1.02)
}

/// The text on a keyboard focus fill reaches 4.5:1 on the fill at rest and
/// under the pointer, on the fallbacks and on the desktop captures.
#[test]
fn the_text_on_a_focus_fill_reads_at_4_5_at_rest_and_under_the_pointer() {
    let captures = [INSTALLED_KDEGLOBALS, BREEZEDARK_COLORS]
        .map(|capture| parse(capture).colors.over(Palette::fallback(Theme::Dark)));

    let palettes = [
        (Theme::Dark, Palette::fallback(Theme::Dark)),
        (Theme::Light, Palette::fallback(Theme::Light)),
        (Theme::Dark, captures[0]),
        (Theme::Dark, captures[1]),
    ];

    for (theme, palette) in palettes {
        let tokens = Tokens::derive(&palette, theme);
        let rest = tokens.focus_fill;
        let hovered = focus_under_pointer(rest);

        for fill in [rest, hovered] {
            let ratio = contrast(tokens.on_focus, fill);
            assert!(
                ratio >= 4.5,
                "{theme:?}: text {:?} on {fill:?} is {ratio:.2}:1, text {:?}, shadow {:?}, window {:?}",
                tokens.on_focus,
                tokens.text,
                tokens.shadow_shade,
                tokens.window
            );
        }
    }
}
