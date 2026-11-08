#![expect(
    clippy::disallowed_methods,
    reason = "tests build scheme palettes from literal colours"
)]

use egui::{Color32, Theme};

use super::super::Tokens;
use super::letters::{fills, grey};
use crate::oklch::{hue_chroma, lightness};
use crate::palette::{Palette, letter_hues};

/// Nord's official colours, a muted scheme.
fn nord() -> Palette {
    Palette {
        window: Color32::from_rgb(0x3b, 0x42, 0x52),
        view: Color32::from_rgb(0x2e, 0x34, 0x40),
        text: Color32::from_rgb(0xd8, 0xde, 0xe9),
        accent: Color32::from_rgb(0x88, 0xc0, 0xd0),
        on_accent: Color32::from_rgb(0x2e, 0x34, 0x40),
        error: Color32::from_rgb(0xbf, 0x61, 0x6a),
        warning: Color32::from_rgb(0xeb, 0xcb, 0x8b),
        success: Color32::from_rgb(0xa3, 0xbe, 0x8c),
        link: Some(Color32::from_rgb(0x88, 0xc0, 0xd0)),
        visited: Some(Color32::from_rgb(0xb4, 0x8e, 0xad)),
        tiles: None,
        ansi: None,
    }
}

/// Dracula's official colours, a vivid scheme.
fn dracula() -> Palette {
    Palette {
        window: Color32::from_rgb(0x28, 0x2a, 0x36),
        view: Color32::from_rgb(0x21, 0x22, 0x2c),
        text: Color32::from_rgb(0xf8, 0xf8, 0xf2),
        accent: Color32::from_rgb(0xbd, 0x93, 0xf9),
        on_accent: Color32::from_rgb(0x28, 0x2a, 0x36),
        error: Color32::from_rgb(0xff, 0x55, 0x55),
        warning: Color32::from_rgb(0xff, 0xb8, 0x6c),
        success: Color32::from_rgb(0x50, 0xfa, 0x7b),
        link: Some(Color32::from_rgb(0x8b, 0xe9, 0xfd)),
        visited: Some(Color32::from_rgb(0xff, 0x79, 0xc6)),
        tiles: None,
        ansi: None,
    }
}

/// Catppuccin Mocha as a scheme of its own, with the link colours the
/// fallback leaves out.
fn mocha() -> Palette {
    Palette {
        link: Some(Color32::from_rgb(0x89, 0xb4, 0xfa)),
        visited: Some(Color32::from_rgb(0xb4, 0xbe, 0xfe)),
        ..Palette::fallback(Theme::Dark)
    }
}

fn softness(palette: &Palette, theme: Theme) -> (f32, f32) {
    let tokens = Tokens::derive(palette, theme);

    mean(&fills(&tokens))
}

fn mean(fills: &[Color32]) -> (f32, f32) {
    let (count, light, chroma) =
        fills
            .iter()
            .fold((0.0_f32, 0.0, 0.0), |(count, light, chroma), fill| {
                (
                    count + 1.0,
                    light + lightness(*fill),
                    chroma + hue_chroma(*fill).1,
                )
            });

    (light / count, chroma / count)
}

#[test]
fn a_muted_scheme_gives_softer_tiles_than_a_vivid_one() {
    let (_, muted) = softness(&nord(), Theme::Dark);
    let (_, vivid) = softness(&dracula(), Theme::Dark);
    let (_, fallback) = mean(&letter_hues(Theme::Dark));

    assert!(
        muted < fallback - 0.01,
        "Nord's tiles at chroma {muted:.3}, the fallback's at {fallback:.3}"
    );
    assert!(
        vivid > fallback + 0.01,
        "Dracula's tiles at chroma {vivid:.3}, the fallback's at {fallback:.3}"
    );
}

#[test]
fn the_fallback_as_a_scheme_keeps_the_fallback_s_softness() {
    let (light, chroma) = softness(&mocha(), Theme::Dark);
    let (fallback_light, fallback_chroma) = mean(&letter_hues(Theme::Dark));

    assert!(
        (light - fallback_light).abs() < 0.01,
        "lightness {light:.3}, the fallback's {fallback_light:.3}"
    );
    assert!(
        (chroma - fallback_chroma).abs() < 0.01,
        "chroma {chroma:.3}, the fallback's {fallback_chroma:.3}"
    );
}

#[test]
fn a_grey_scheme_keeps_the_fallback_s_chroma() {
    let (_, chroma) = softness(&grey(), Theme::Dark);
    let (_, fallback) = mean(&letter_hues(Theme::Dark));

    assert!(
        (chroma - fallback).abs() < 0.01,
        "chroma {chroma:.3}, the fallback's {fallback:.3}"
    );
}
