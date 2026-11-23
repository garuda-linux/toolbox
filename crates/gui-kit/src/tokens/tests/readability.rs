use egui::Color32;

use super::super::contrast;
use super::super::readability::{Ends, readable};
use crate::oklch::{hue_chroma, lightness};

fn grey(level: u8) -> Color32 {
    Color32::from_rgb(level, level, level)
}

#[test]
fn the_light_end_is_the_lighter_of_text_and_window() {
    // Darkly's white: the text in a dark theme, the
    // window in a light one.
    let dark = Ends {
        text: grey(0xcd),
        window: grey(0x1e),
        shadow: grey(0x05),
    };

    let light = Ends {
        text: grey(0x4c),
        window: grey(0xef),
        shadow: grey(0x05),
    };

    assert_eq!(dark.light(), dark.text);
    assert_eq!(light.light(), light.window);
}

#[test]
fn readable_reaches_the_ratio_from_the_shadow_when_text_and_window_cannot() {
    // Text and window both sit near the grey background, so only the black
    // shadow shade can carry the colour to 4.5:1.
    let ends = Ends {
        text: grey(0x80),
        window: grey(0x88),
        shadow: grey(0x00),
    };

    let background = grey(0x80);

    let fitted = readable(grey(0x80), background, 4.5, ends);

    assert!(contrast(fitted, background) >= 4.5, "{fitted:?}");
    assert_ne!(fitted, grey(0x80));
}

#[test]
fn readable_lifts_the_lightness_when_no_end_reaches_the_ratio() {
    // Solarized Dark's text is its lightest end and reads 4.6:1 on its card,
    // so only a lighter text of the same hue reaches 7:1.
    let text = Color32::from_rgb(0x83, 0x94, 0x96);
    let ends = Ends {
        text,
        window: Color32::from_rgb(0x00, 0x2b, 0x36),
        shadow: grey(0x00),
    };

    let card = Color32::from_rgb(0x02, 0x2e, 0x3a);

    let fitted = readable(text, card, 7.0, ends);

    assert!(contrast(fitted, card) >= 7.0, "{fitted:?}");
    assert!(lightness(fitted) > lightness(text), "{fitted:?}");
    let (hue, _) = hue_chroma(fitted);
    let (was, _) = hue_chroma(text);
    assert!((hue - was).abs() < 3.0, "{hue:.1} from {was:.1}");
}

#[test]
fn readable_darkens_a_light_ink_its_ends_cannot_carry() {
    // White misses 4.5:1 on Ayu Light's orange accent, so the ink takes the
    // lightness of a dark colour.
    let ends = Ends {
        text: Color32::from_rgb(0x5c, 0x67, 0x73),
        window: Color32::from_rgb(0xfa, 0xfa, 0xfa),
        shadow: Color32::from_rgb(0x5c, 0x5c, 0x5c),
    };

    let accent = Color32::from_rgb(0xff, 0x6a, 0x00);

    let fitted = readable(ends.window, accent, 4.5, ends);

    assert!(contrast(fitted, accent) >= 4.5, "{fitted:?}");
}

#[test]
fn readable_keeps_the_ends_move_where_an_end_reaches_the_ratio() {
    let ends = Ends {
        text: grey(0xff),
        window: grey(0x20),
        shadow: grey(0x00),
    };

    let fitted = readable(grey(0x90), grey(0x20), 7.0, ends);

    assert!(contrast(fitted, grey(0x20)) >= 7.0);
    let (_, chroma) = hue_chroma(fitted);
    assert!(chroma < 0.001, "a grey stays grey: {fitted:?}");
}
