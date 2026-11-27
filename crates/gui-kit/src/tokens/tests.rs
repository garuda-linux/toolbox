#![expect(
    clippy::disallowed_methods,
    reason = "tests build desktop palettes from literal colours"
)]

mod cards;
mod code;
mod darkly;
mod frames;
mod frost;
mod glass;
mod kirigami;
mod letter_softness;
mod letters;
mod lights;
mod readability;
mod scrim;
mod shade;
mod sidebar;
mod terminal_glass;
mod tiles;
mod toolbar;

use egui::{Color32, Theme};

use super::{Tokens, contrast};
use crate::palette::Palette;

pub(super) fn lightness(color: Color32) -> u16 {
    u16::from(color.r()) + u16::from(color.g()) + u16::from(color.b())
}

fn with_accent(theme: Theme, accent: Color32) -> Tokens {
    let mut palette = Palette::fallback(theme);
    palette.accent = accent;
    Tokens::derive(&palette, theme)
}

#[test]
fn the_button_steps_are_darklys_lighter_hover_and_darker_press() {
    // Darkly's button hover and press (darklyhelper.cpp:848). The fallback
    // names no focus, so the accent stands in. The hover is not checked
    // against the window, as a dark Button can round to its grey.
    let palette = Palette::fallback(Theme::Dark);
    let tokens = Tokens::derive(&palette, Theme::Dark);

    assert_eq!(
        tokens.button_hover,
        super::darkly::lighter(tokens.input, 1.05)
    );
    assert_eq!(
        tokens.button_pressed,
        super::darkly::darker(palette.accent, 1.1)
    );
}

#[test]
fn light_sidebar_is_the_window_under_the_text_coloured_scrim() {
    // The header's white scrim is the text colour,
    // so the light sidebar sits below the window, not above it.
    let tokens = Tokens::derive(&Palette::fallback(Theme::Light), Theme::Light);

    assert!(lightness(tokens.sidebar) < lightness(tokens.window));
}

#[test]
fn text_stays_readable_on_a_low_contrast_desktop_theme() {
    let mut washed = Palette::fallback(Theme::Light);
    washed.text = Color32::from_rgb(0x9a, 0x9c, 0xa8);
    washed.success = Color32::from_rgb(0xb0, 0xe0, 0xb0);

    let tokens = Tokens::derive(&washed, Theme::Light);

    assert!(contrast(tokens.text, tokens.card) >= 7.0);
    assert!(contrast(tokens.secondary, tokens.card) >= 4.5);
    assert!(contrast(tokens.dim, tokens.card) >= 3.0);
    assert!(contrast(tokens.success, tokens.card) >= 4.5);
}

#[test]
fn the_desktop_decides_the_accent() {
    let mut desktop = Palette::fallback(Theme::Dark);
    desktop.accent = Color32::from_rgb(0x3d, 0xae, 0xe9);

    assert_eq!(Tokens::derive(&desktop, Theme::Dark).accent, desktop.accent);
}

#[test]
fn black_on_white_is_twenty_one_to_one() {
    let ratio = contrast(Color32::BLACK, Color32::WHITE);

    assert!((ratio - 21.0).abs() < 0.01, "{ratio}");
}

#[test]
fn text_on_the_selection_tint_stays_readable() {
    for theme in [Theme::Dark, Theme::Light] {
        let tokens = Tokens::derive(&Palette::fallback(theme), theme);

        let ratio = contrast(tokens.on_tint, tokens.accent_tint);
        assert!(ratio >= 4.5, "{theme:?}: {ratio}");
    }
}

#[test]
fn a_pale_desktop_accent_takes_the_text_for_its_selection() {
    // The desktop's light selection text misses 3:1 on a pale accent, so the
    // theme's text, which reads better on it, takes its place.
    let tokens = with_accent(Theme::Light, Color32::from_rgb(0xb8, 0xd8, 0xf8));

    assert_eq!(tokens.on_tint, tokens.text);
}

#[test]
fn the_selection_tint_stands_out_from_the_sidebar_in_both_themes() {
    for theme in [Theme::Dark, Theme::Light] {
        let tokens = Tokens::derive(&Palette::fallback(theme), theme);
        let ratio = contrast(tokens.accent_tint, tokens.sidebar);

        assert!(ratio >= 3.0, "{theme:?}: {ratio}");
    }
}

#[test]
fn a_soft_button_on_a_dark_dialog_keeps_the_field_fill() {
    let tokens = Tokens::derive(&Palette::fallback(Theme::Dark), Theme::Dark);

    // Breeze paints a button's Button fill on any window-coloured surface, so
    // a dialog's Soft button keeps it.
    assert_eq!(tokens.field_on_raised, tokens.field);

    assert!(
        lightness(tokens.field_on_raised) < lightness(tokens.button_hover),
        "a hover still shows a step"
    );
}

#[test]
fn a_soft_button_on_a_white_dialog_keeps_the_field_fill() {
    let tokens = Tokens::derive(&Palette::fallback(Theme::Light), Theme::Light);

    assert_eq!(tokens.field_on_raised, tokens.field);
}

#[test]
fn a_rim_reads_on_the_field_fill() {
    for theme in [Theme::Dark, Theme::Light] {
        let tokens = Tokens::derive(&Palette::fallback(theme), theme);

        let ratio = contrast(tokens.rim, tokens.field);
        assert!(ratio >= 3.0, "{theme:?}: {ratio}");
    }
}

pub(super) fn accents() -> [(Theme, Color32); 4] {
    [
        (Theme::Light, Color32::from_rgb(0xb8, 0xd8, 0xf8)),
        (Theme::Light, Color32::from_rgb(0x3d, 0xae, 0xe9)),
        (Theme::Dark, Color32::from_rgb(0x1d, 0x4e, 0xd8)),
        (Theme::Dark, Color32::from_rgb(0x3d, 0xae, 0xe9)),
    ]
}

pub(super) fn every_tokens() -> Vec<(Theme, Tokens)> {
    let fallback = [Theme::Dark, Theme::Light]
        .map(|theme| (theme, Tokens::derive(&Palette::fallback(theme), theme)));

    let picked = accents().map(|(theme, accent)| (theme, with_accent(theme, accent)));

    fallback.into_iter().chain(picked).collect()
}

/// The accent line is the accent itself, with no 3:1 move. A pale accent
/// therefore reads faintly on a light fill.
#[test]
fn the_accent_ring_is_the_accent_unmoved() {
    for (theme, accent) in accents() {
        let tokens = with_accent(theme, accent);
        assert_eq!(
            tokens.accent_ring, accent,
            "{theme:?}: the accent ring is the accent"
        );
    }
}

#[test]
fn accent_text_reaches_four_and_a_half_to_one_with_any_accent() {
    for (theme, accent) in accents() {
        let tokens = with_accent(theme, accent);

        for fill in [tokens.window, tokens.card, tokens.accent_soft] {
            let ratio = contrast(tokens.accent_text, fill);
            assert!(
                ratio >= 4.5,
                "{theme:?} {accent:?}: accent text at {ratio:.2} on {fill:?}"
            );
        }
    }
}

fn breeze_light() -> Palette {
    let mut breeze = Palette::fallback(Theme::Light);
    breeze.window = Color32::from_rgb(0xef, 0xf0, 0xf1);
    breeze.view = Color32::from_rgb(0xff, 0xff, 0xff);
    breeze.text = Color32::from_rgb(0x23, 0x26, 0x29);
    breeze.warning = Color32::from_rgb(0xf6, 0x74, 0x00);
    breeze.error = Color32::from_rgb(0xda, 0x44, 0x53);
    breeze.success = Color32::from_rgb(0x27, 0xae, 0x60);
    breeze
}

#[test]
fn status_and_secondary_text_stay_readable_on_every_fill_they_sit_on() {
    let cases = [
        (Palette::fallback(Theme::Dark), Theme::Dark),
        (Palette::fallback(Theme::Light), Theme::Light),
        (breeze_light(), Theme::Light),
    ];

    for (palette, theme) in cases {
        let tokens = Tokens::derive(&palette, theme);
        let fills = [
            tokens.card,
            tokens.window,
            tokens.field,
            tokens.button_hover,
            tokens.sidebar,
        ];

        let inks = [
            ("secondary", tokens.secondary),
            ("warning", tokens.warning),
            ("error", tokens.error),
            ("success", tokens.success),
        ];

        for (name, ink) in inks {
            for fill in fills {
                let ratio = contrast(ink, fill);
                assert!(
                    ratio >= 4.5,
                    "{theme:?}: {name} reaches only {ratio:.2}:1 on {fill:?}"
                );
            }
        }
    }
}

#[test]
fn the_active_text_is_the_desktops_active_colour_and_the_accent_without_one() {
    let mut palette = Palette::fallback(Theme::Dark);
    assert_eq!(
        Tokens::derive(&palette, Theme::Dark).roles.active,
        Tokens::derive(&palette, Theme::Dark).accent
    );

    let peach = Color32::from_rgb(250, 179, 135);
    palette.active = Some(peach);
    assert_eq!(Tokens::derive(&palette, Theme::Dark).roles.active, peach);
}

#[test]
fn the_notes_take_the_complementary_set_and_the_window_without_one() {
    let mut palette = Palette::fallback(Theme::Dark);
    let tokens = Tokens::derive(&palette, Theme::Dark);
    assert_eq!(tokens.roles.complementary, tokens.window);

    let note = Color32::from_rgb(0x31, 0x36, 0x3b);
    palette.complementary = Some(note);
    assert_eq!(
        Tokens::derive(&palette, Theme::Dark).roles.complementary,
        note
    );
}

/// Whether `scrim` is `colour` at the scrim's own alpha. egui stores the
/// colour premultiplied, so the check compares it premultiplied, to one level.
pub(super) fn tinted_as(scrim: Color32, colour: Color32) -> bool {
    let expected = colour.gamma_multiply(f32::from(scrim.a()) / 255.0);
    [
        (scrim.r(), expected.r()),
        (scrim.g(), expected.g()),
        (scrim.b(), expected.b()),
    ]
    .iter()
    .all(|(got, want)| got.abs_diff(*want) <= 1)
}
