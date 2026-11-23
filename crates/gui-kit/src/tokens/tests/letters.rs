#![expect(
    clippy::disallowed_methods,
    reason = "tests build desktop palettes from literal colours"
)]

use egui::{Color32, Theme};

use super::super::letters::{LETTER_HUES, gap};
use super::super::{Tokens, contrast, mix};
use crate::oklch::{hue_chroma, lightness};
use crate::palette::{Palette, letter_hues};

/// Letter tiles that stay under 4.5:1. Each fill darkens only half way toward
/// the shadow shade, and these hues stop short of it, so each floor is the
/// ratio that hue reaches, to one decimal.
const LETTER_FLOORS: [(&str, Theme, usize, f32); 10] = [
    ("fallback", Theme::Dark, 0, 4.3),
    ("fallback", Theme::Dark, 1, 4.2),
    ("fallback", Theme::Dark, 2, 4.1),
    ("fallback", Theme::Dark, 3, 3.9),
    ("fallback", Theme::Dark, 4, 3.9),
    ("fallback", Theme::Dark, 5, 4.1),
    ("fallback", Theme::Dark, 6, 4.2),
    ("fallback", Theme::Dark, 7, 4.3),
    ("fallback", Theme::Light, 3, 4.4),
    ("fallback", Theme::Light, 4, 4.4),
];

fn breeze_dark() -> Palette {
    Palette {
        window: Color32::from_rgb(32, 35, 38),
        view: Color32::from_rgb(20, 22, 24),
        button: None,
        text: Color32::from_rgb(252, 252, 252),
        accent: Color32::from_rgb(61, 174, 233),
        on_accent: Color32::from_rgb(252, 252, 252),
        error: Color32::from_rgb(218, 68, 83),
        warning: Color32::from_rgb(246, 116, 0),
        success: Color32::from_rgb(39, 174, 96),
        link: Some(Color32::from_rgb(29, 153, 243)),
        visited: Some(Color32::from_rgb(155, 89, 182)),
        active: None,
        complementary: None,
        complementary_text: None,
        decoration_hover: None,
        decoration_focus: None,
        tooltip: None,
        tooltip_text: None,
        contrast: None,
        tiles: None,
        ansi: None,
    }
}

fn breeze_light() -> Palette {
    Palette {
        window: Color32::from_rgb(239, 240, 241),
        view: Color32::from_rgb(255, 255, 255),
        button: None,
        text: Color32::from_rgb(35, 38, 41),
        link: Some(Color32::from_rgb(41, 128, 185)),
        ..breeze_dark()
    }
}

/// A scheme whose every colour is one blue, so the tiles fill up with
/// turns of the accent.
fn one_hue() -> Palette {
    let blue = Color32::from_rgb(0x1e, 0x66, 0xf5);
    Palette {
        accent: blue,
        error: blue,
        warning: blue,
        success: blue,
        link: Some(blue),
        active: None,
        complementary: None,
        complementary_text: None,
        decoration_hover: None,
        decoration_focus: None,
        tooltip: None,
        tooltip_text: None,
        contrast: None,
        visited: Some(blue),
        ..Palette::fallback(Theme::Light)
    }
}

/// A grey accent and grey status colours, whose hue says nothing.
pub(super) fn grey() -> Palette {
    let grey = Color32::from_rgb(0x80, 0x80, 0x80);
    Palette {
        accent: grey,
        error: grey,
        warning: grey,
        success: grey,
        ..Palette::fallback(Theme::Dark)
    }
}

fn cases() -> [(&'static str, Theme, Tokens); 7] {
    [
        ("fallback", Palette::fallback(Theme::Dark), Theme::Dark),
        ("fallback", Palette::fallback(Theme::Light), Theme::Light),
        ("breeze dark", breeze_dark(), Theme::Dark),
        ("breeze light", breeze_light(), Theme::Light),
        ("one hue", one_hue(), Theme::Light),
        ("one hue", one_hue(), Theme::Dark),
        ("grey", grey(), Theme::Dark),
    ]
    .map(|(name, palette, theme)| (name, theme, Tokens::derive(&palette, theme)))
}

pub(super) fn fills(tokens: &Tokens) -> Vec<Color32> {
    (0..LETTER_HUES)
        .map(|index| tokens.tiles.letters().of(index).fill)
        .collect()
}

fn hue_angles(tokens: &Tokens) -> Vec<f32> {
    fills(tokens)
        .into_iter()
        .map(|fill| hue_chroma(fill).0)
        .collect()
}

#[test]
fn every_letter_reads_at_four_and_a_half_to_one_on_its_tile() {
    let mut below_target = Vec::new();
    for (name, theme, tokens) in cases() {
        for index in 0..LETTER_HUES {
            let tile = tokens.tiles.letters().of(index);
            let ratio = contrast(tile.ink, tile.fill);
            if ratio >= 4.5 {
                continue;
            }

            let floor = LETTER_FLOORS
                .iter()
                .find(|(entry, entry_theme, entry_hue, _)| {
                    *entry == name && *entry_theme == theme && *entry_hue == index
                })
                .map(|(_, _, _, floor)| *floor);

            match floor {
                Some(floor) => {
                    assert!(ratio >= floor, "{name} {theme:?} hue {index}: {ratio:.2}:1");
                }
                None => below_target.push(format!("{name} {theme:?} hue {index}: {ratio:.2}:1")),
            }
        }
    }

    assert!(below_target.is_empty(), "{below_target:#?}");
}

#[test]
fn the_letter_is_the_text_or_window_that_reads_better_on_its_tile() {
    // Each tile takes whichever of the theme text and window reads better on
    // its own fill.
    for (name, theme, tokens) in cases() {
        for index in 0..LETTER_HUES {
            let tile = tokens.tiles.letters().of(index);
            let better = if contrast(tokens.text, tile.fill) >= contrast(tokens.window, tile.fill) {
                tokens.text
            } else {
                tokens.window
            };

            assert_eq!(tile.ink, better, "{name} {theme:?} hue {index}");
        }
    }
}

#[test]
fn the_fills_are_eight_hues_set_apart() {
    assert_eq!(LETTER_HUES, 8);

    for (name, theme, tokens) in cases() {
        let hues = hue_angles(&tokens);

        for (index, hue) in hues.iter().enumerate() {
            for other in hues.iter().skip(index + 1) {
                assert!(
                    gap(*hue, *other) >= 12.0,
                    "{name} {theme:?}: hues {hue:.0} and {other:.0} are too close"
                );
            }
        }
    }
}

#[test]
fn a_letter_tile_stands_out_from_the_card_of_its_row() {
    for (name, theme, tokens) in cases() {
        for (index, fill) in fills(&tokens).into_iter().enumerate() {
            let ratio = contrast(fill, tokens.card);
            assert!(ratio >= 1.5, "{name} {theme:?} hue {index}: {ratio:.2}:1");
        }
    }
}

#[test]
fn without_a_desktop_scheme_the_fallback_hues_take_the_shadow_darkening() {
    // The fallback's tiles take the same darkening as a scheme's: each hue is
    // kept or moved toward the shadow shade by at most half way, never
    // lightened.
    for theme in [Theme::Dark, Theme::Light] {
        let tokens = Tokens::derive(&Palette::fallback(theme), theme);

        for (shown, hue) in fills(&tokens).into_iter().zip(letter_hues(theme)) {
            assert!(
                lightness(shown) <= lightness(hue) + 0.001,
                "{theme:?}: {shown:?} is lighter than {hue:?}"
            );
            let darkest = mix(hue, tokens.shadow_shade, 0.5);
            assert!(
                lightness(shown) >= lightness(darkest) - 0.001,
                "{theme:?}: {shown:?} is darker than half way to the shadow"
            );
        }
    }
}

#[test]
fn the_scheme_s_own_colours_lead_the_hues() {
    let palette = breeze_dark();
    let tokens = Tokens::derive(&palette, Theme::Dark);
    let hues = hue_angles(&tokens);

    let named = [
        ("accent", palette.accent),
        ("positive", palette.success),
        ("neutral", palette.warning),
        ("negative", palette.error),
    ];

    for (index, (role, colour)) in named.into_iter().enumerate() {
        let (wanted, _) = hue_chroma(colour);
        let hue = hues.get(index).copied().unwrap_or_default();
        assert!(
            gap(hue, wanted) <= 6.0,
            "tile {index} is at {hue:.0}, the {role} colour at {wanted:.0}"
        );
    }
}

#[test]
fn another_scheme_gives_other_fills() {
    let dark = Tokens::derive(&breeze_dark(), Theme::Dark);
    let fallback = Tokens::derive(&Palette::fallback(Theme::Dark), Theme::Dark);

    assert_ne!(fills(&dark), fills(&fallback));
}

#[test]
fn an_index_past_the_palette_wraps_round() {
    let (_, _, tokens) = cases()[0];
    assert_eq!(
        tokens.tiles.letters().of(LETTER_HUES + 2),
        tokens.tiles.letters().of(2)
    );
}

/// Ayu Light, whose near-white window gives KDE's mid-grey shadow shade.
fn ayu_light() -> Palette {
    Palette {
        window: Color32::from_rgb(0xfa, 0xfa, 0xfa),
        view: Color32::from_rgb(0xf3, 0xf3, 0xf3),
        text: Color32::from_rgb(0x5c, 0x67, 0x73),
        accent: Color32::from_rgb(0xff, 0x6a, 0x00),
        on_accent: Color32::from_rgb(0xfa, 0xfa, 0xfa),
        error: Color32::from_rgb(0xff, 0x33, 0x33),
        warning: Color32::from_rgb(0xe6, 0xb6, 0x73),
        success: Color32::from_rgb(0x86, 0xb3, 0x00),
        link: Some(Color32::from_rgb(0x36, 0xa3, 0xd9)),
        visited: Some(Color32::from_rgb(0xa3, 0x7a, 0xcc)),
        ..breeze_dark()
    }
}

#[test]
fn a_shadow_shade_too_light_to_carry_the_letter_leaves_the_fill_to_its_lightness() {
    // Half way to a mid-grey shadow shade stops short of 4.5:1, so the fill
    // takes a darker lightness of its own hue.
    let tokens = Tokens::derive(&ayu_light(), Theme::Light);

    for index in 0..LETTER_HUES {
        let tile = tokens.tiles.letters().of(index);
        let ratio = contrast(tile.ink, tile.fill);
        assert!(ratio >= 4.5, "hue {index}: {ratio:.2}:1");
        assert!(contrast(tile.fill, tokens.card) >= 1.5, "hue {index}");
    }
}
