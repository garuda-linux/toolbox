use egui::{Color32, Theme};

use super::super::darkly::{self, is_dark_theme, lighter};
use crate::palette::Palette;
use crate::tokens::{Tokens, contrast, mix};

fn channels(colour: Color32) -> [u8; 3] {
    [colour.r(), colour.g(), colour.b()]
}

fn within_one(colour: Color32, expected: [u8; 3]) -> bool {
    channels(colour)
        .iter()
        .zip(expected)
        .all(|(&got, want)| got.abs_diff(want) <= 1)
}

fn dark() -> Palette {
    Palette::fallback(Theme::Dark)
}

fn light() -> Palette {
    Palette::fallback(Theme::Light)
}

#[test]
fn highlighted_text_is_the_desktops_selection_text_where_it_reads_on_the_accent() {
    // A HighlightedText at 4.5:1 or more on the accent keeps its colour.
    let palette = dark();
    let tokens = Tokens::derive(&palette, Theme::Dark);

    assert_eq!(tokens.highlighted_text, palette.on_accent);
    assert_eq!(tokens.on_tint, palette.on_accent);
}

#[test]
fn highlighted_text_takes_the_window_or_the_text_below_four_and_a_half_to_one() {
    // A text under 4.5:1 on the accent gives way to whichever of the theme's
    // text and window reads better on it, here the window.
    let mut palette = dark();
    palette.on_accent = Color32::from_rgb(0x88, 0x88, 0x88);
    let tokens = Tokens::derive(&palette, Theme::Dark);

    assert_eq!(tokens.highlighted_text, tokens.window);
    assert_eq!(tokens.on_tint, tokens.highlighted_text);
}

#[test]
fn highlighted_text_none_of_the_ends_carries_is_fitted_to_four_and_a_half() {
    // Ayu Light's light selection text and its grey text both miss 4.5:1 on
    // its orange accent, so the better of them is fitted until it reads.
    let mut palette = light();
    palette.window = Color32::from_rgb(0xfa, 0xfa, 0xfa);
    palette.view = Color32::from_rgb(0xf3, 0xf3, 0xf3);
    palette.text = Color32::from_rgb(0x5c, 0x67, 0x73);
    palette.accent = Color32::from_rgb(0xff, 0x6a, 0x00);
    palette.on_accent = palette.window;
    let tokens = Tokens::derive(&palette, Theme::Light);

    let ratio = contrast(tokens.on_tint, tokens.accent_tint);
    assert!(ratio >= 4.5, "{ratio:.2}:1");
    assert_eq!(tokens.on_tint, tokens.highlighted_text);
}

#[test]
fn sidebar_text_is_the_full_text_in_both_themes() {
    for (palette, theme) in [(dark(), Theme::Dark), (light(), Theme::Light)] {
        let tokens = Tokens::derive(&palette, theme);
        assert_eq!(tokens.sidebar_text, tokens.text);
    }
}

#[test]
fn separator_is_the_text_at_16_on_a_dark_window_and_the_shadow_shade_at_16_on_a_light_one() {
    let dark_tokens = Tokens::derive(&dark(), Theme::Dark);
    let light_tokens = Tokens::derive(&light(), Theme::Light);

    assert_eq!(dark_tokens.separator, darkly::alpha(dark_tokens.text, 16));
    assert_eq!(
        light_tokens.separator,
        darkly::alpha(light_tokens.shadow_shade, 16)
    );
}

#[test]
fn grey_110_is_a_dark_theme_and_111_is_a_light_one() {
    assert!(is_dark_theme(Color32::from_rgb(110, 110, 110)));
    assert!(!is_dark_theme(Color32::from_rgb(111, 111, 111)));
}

#[test]
fn sidebar_edge_is_the_shadow_shade_at_50_on_a_dark_window_and_20_on_a_light_one() {
    let dark_tokens = Tokens::derive(&dark(), Theme::Dark);
    let light_tokens = Tokens::derive(&light(), Theme::Light);

    assert_eq!(
        dark_tokens.sidebar_edge,
        darkly::alpha(dark_tokens.shadow_shade, 50)
    );
    assert_eq!(
        light_tokens.sidebar_edge,
        darkly::alpha(light_tokens.shadow_shade, 20)
    );
}

#[test]
fn control_shadow_is_the_shadow_shade_at_40_of_255() {
    let tokens = Tokens::derive(&dark(), Theme::Dark);

    assert_eq!(
        tokens.control_shadow,
        darkly::alpha(tokens.shadow_shade, 40)
    );
}

#[test]
fn card_is_the_window_mixed_toward_the_view_by_0_3() {
    let mut palette = dark();
    palette.window = Color32::from_rgb(32, 35, 38);
    palette.view = Color32::from_rgb(20, 22, 24);
    let tokens = Tokens::derive(&palette, Theme::Dark);

    assert_eq!(
        tokens.card,
        mix(palette.window, palette.view, 0.3),
        "Darkly's frameBackgroundColor (darklyhelper.cpp:180)"
    );
}

#[test]
fn button_hover_is_the_input_fill_lightened_by_5_percent() {
    let tokens = Tokens::derive(&dark(), Theme::Dark);

    assert_eq!(tokens.button_hover, lighter(tokens.input, 1.05));
}

#[test]
fn button_pressed_is_the_focus_colour_darkened_by_10_percent() {
    // With no focus colour named, the focus is the accent, Catppuccin's
    // mauve (203, 166, 247), darkened by 110.
    let tokens = Tokens::derive(&dark(), Theme::Dark);

    assert!(
        within_one(tokens.button_pressed, [185, 151, 225]),
        "{:?}",
        channels(tokens.button_pressed)
    );
}

#[test]
fn default_button_is_the_input_fill_mixed_toward_the_view_by_0_7() {
    let button = Color32::from_rgb(50, 50, 50);
    let view = Color32::from_rgb(44, 44, 44);
    let mut palette = dark();
    palette.button = Some(button);
    palette.view = view;
    let tokens = Tokens::derive(&palette, Theme::Dark);

    assert_eq!(tokens.default_button, mix(tokens.input, view, 0.7));
    assert_ne!(tokens.default_button, mix(button, view, 0.7));
}

#[test]
fn item_hover_is_the_accent_at_a_fifth_of_its_opacity() {
    let mut palette = dark();
    palette.accent = Color32::from_rgb(27, 145, 213);
    let tokens = Tokens::derive(&palette, Theme::Dark);

    // Qt's setAlphaF(0.2) gives 51 of 255 (darklystyle.cpp:4768), and egui
    // stores the fill premultiplied, as every tinted fill is.
    assert_eq!(tokens.item_hover.a(), 51);
    assert_eq!(tokens.item_hover, palette.accent.gamma_multiply(0.2));
}

#[test]
fn tint_hover_is_the_selection_lightened_by_10_percent() {
    let mut palette = dark();
    palette.accent = Color32::from_rgb(27, 145, 213);
    let tokens = Tokens::derive(&palette, Theme::Dark);

    assert_eq!(tokens.tint_hover, lighter(tokens.accent_tint, 1.1));
    assert!(within_one(tokens.tint_hover, [30, 160, 234]));
}

#[test]
fn hover_focus_and_tooltip_take_the_accent_card_and_text_without_their_own_roles() {
    for (palette, theme) in [(dark(), Theme::Dark), (light(), Theme::Light)] {
        let tokens = Tokens::derive(&palette, theme);

        assert_eq!(tokens.decoration_hover, tokens.accent, "{theme:?}");
        assert_eq!(tokens.decoration_focus, tokens.accent, "{theme:?}");
        assert_eq!(tokens.tooltip, tokens.raised, "{theme:?}");
        assert_eq!(tokens.tooltip_text, tokens.text, "{theme:?}");
    }
}

#[test]
fn the_desktops_own_hover_focus_and_tooltip_are_used() {
    let mut palette = dark();
    palette.decoration_hover = Some(Color32::from_rgb(10, 20, 30));
    palette.decoration_focus = Some(Color32::from_rgb(40, 50, 60));
    palette.tooltip = Some(Color32::from_rgb(70, 80, 90));
    let tokens = Tokens::derive(&palette, Theme::Dark);

    assert_eq!(tokens.decoration_hover, Color32::from_rgb(10, 20, 30));
    assert_eq!(tokens.decoration_focus, Color32::from_rgb(40, 50, 60));
    assert_eq!(tokens.tooltip, Color32::from_rgb(70, 80, 90));
}

#[test]
fn lighter_at_110_scales_the_value_and_keeps_grey_grey() {
    assert!(within_one(
        lighter(Color32::from_rgb(100, 100, 100), 1.1),
        [110, 110, 110]
    ));
    assert!(within_one(
        lighter(Color32::from_rgb(27, 145, 213), 1.1),
        [30, 160, 234]
    ));
}

#[test]
fn lighter_past_full_value_clamps_to_white_and_desaturates() {
    assert_eq!(
        lighter(Color32::from_rgb(240, 240, 240), 1.1),
        Color32::from_rgb(255, 255, 255)
    );
    let bright = lighter(Color32::from_rgb(240, 200, 100), 1.1);
    assert_eq!(bright.r(), 255);
    assert!(bright.g() > 200 && bright.b() > 100, "{bright:?}");
}

#[test]
fn darker_at_110_scales_the_value_down() {
    assert!(within_one(
        darkly::darker(Color32::from_rgb(52, 120, 218), 1.1),
        [47, 109, 198]
    ));
}

#[test]
fn highlighted_text_between_three_and_four_and_a_half_is_fitted() {
    // Solarized Light's base3 reads 3.4:1 on its blue accent, readable for a
    // tick but not for the label of a selected row.
    let mut palette = light();
    palette.window = Color32::from_rgb(0xfd, 0xf6, 0xe3);
    palette.view = Color32::from_rgb(0xee, 0xe8, 0xd5);
    palette.text = Color32::from_rgb(0x65, 0x7b, 0x83);
    palette.accent = Color32::from_rgb(0x26, 0x8b, 0xd2);
    palette.on_accent = palette.window;
    let tokens = Tokens::derive(&palette, Theme::Light);

    let ratio = contrast(tokens.on_tint, tokens.accent_tint);
    assert!(ratio >= 4.5, "{ratio:.2}:1");
}
