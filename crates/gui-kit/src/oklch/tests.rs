#![expect(
    clippy::disallowed_methods,
    reason = "the tests build the colours they check"
)]

use egui::Color32;

use super::{from_oklch, hue_chroma, lightness, shift_hue, with_lightness};

const VIOLET: Color32 = Color32::from_rgb(0x7f, 0x40, 0xc0);

fn close(first: Color32, second: Color32, tolerance: u8) -> bool {
    [
        (first.r(), second.r()),
        (first.g(), second.g()),
        (first.b(), second.b()),
    ]
    .iter()
    .all(|(one, other)| one.abs_diff(*other) <= tolerance)
}

#[test]
fn a_zero_shift_keeps_the_colour() {
    assert!(close(shift_hue(VIOLET, 0.0), VIOLET, 1));
}

#[test]
fn a_shift_turns_the_hue_and_keeps_the_chroma_about_right() {
    let (hue, chroma) = hue_chroma(VIOLET);
    let (turned, turned_chroma) = hue_chroma(shift_hue(VIOLET, 150.0));

    let wanted = (hue + 150.0).rem_euclid(360.0);
    let off = (turned - wanted).rem_euclid(360.0);
    assert!(
        off.min(360.0 - off) < 4.0,
        "hue moved to {turned}, wanted {wanted}"
    );
    assert!(
        turned_chroma <= chroma + 0.01,
        "chroma grew to {turned_chroma}"
    );
}

#[test]
fn a_grey_stays_grey() {
    let grey = Color32::from_rgb(0x80, 0x80, 0x80);
    let shifted = shift_hue(grey, 120.0);
    assert!(shifted.r().abs_diff(shifted.g()) <= 1 && shifted.g().abs_diff(shifted.b()) <= 1);
}

#[test]
fn the_alpha_is_kept() {
    assert_eq!(shift_hue(VIOLET.gamma_multiply(0.5), 40.0).a(), 128);
}

#[test]
fn a_new_lightness_keeps_the_hue_and_the_chroma() {
    let (hue, chroma) = hue_chroma(VIOLET);
    let lighter = with_lightness(VIOLET, lightness(VIOLET) + 0.1);
    let (moved, moved_chroma) = hue_chroma(lighter);

    assert!(
        (lightness(lighter) - lightness(VIOLET) - 0.1).abs() < 0.01,
        "lightness {}",
        lightness(lighter)
    );
    assert!((moved - hue).abs() < 2.0, "hue moved to {moved}");
    assert!(
        (moved_chroma - chroma).abs() < 0.01,
        "chroma {moved_chroma}"
    );
}

#[test]
fn a_lightness_beyond_srgb_gives_up_chroma_not_hue() {
    let (hue, _) = hue_chroma(VIOLET);
    let pale = with_lightness(VIOLET, 0.95);
    let (moved, _) = hue_chroma(pale);

    assert!((lightness(pale) - 0.95).abs() < 0.01);
    assert!((moved - hue).abs() < 4.0, "hue moved to {moved}");
}

#[test]
fn a_colour_from_its_lightness_chroma_and_hue_is_that_colour() {
    let (hue, chroma) = hue_chroma(VIOLET);
    assert!(close(from_oklch(lightness(VIOLET), chroma, hue), VIOLET, 1));
}

#[test]
fn a_chroma_beyond_srgb_is_reduced_at_its_hue_and_lightness() {
    let teal = from_oklch(0.5, 0.3, 190.0);
    let (hue, chroma) = hue_chroma(teal);

    assert!((lightness(teal) - 0.5).abs() < 0.01);
    assert!((hue - 190.0).abs() < 4.0, "hue moved to {hue}");
    assert!(chroma < 0.3, "chroma {chroma}");
}
