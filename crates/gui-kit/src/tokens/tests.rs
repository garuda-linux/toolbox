#![expect(
    clippy::disallowed_methods,
    reason = "tests build desktop palettes from literal colours"
)]

mod letter_softness;
mod letters;
mod scrim;
mod sidebar;
mod terminal_glass;
mod tiles;

use egui::{Color32, Theme};

use super::{Tokens, contrast};
use crate::palette::Palette;

fn lightness(color: Color32) -> u16 {
    u16::from(color.r()) + u16::from(color.g()) + u16::from(color.b())
}

fn with_accent(theme: Theme, accent: Color32) -> Tokens {
    let mut palette = Palette::fallback(theme);
    palette.accent = accent;
    Tokens::derive(&palette, theme)
}

#[test]
fn dark_surfaces_rise_from_sidebar_to_pressed() {
    let tokens = Tokens::derive(&Palette::fallback(Theme::Dark), Theme::Dark);

    // The sidebar is the window under the header's black scrim.
    let steps = [
        tokens.sidebar,
        tokens.window,
        tokens.card,
        tokens.field,
        tokens.hover,
        tokens.pressed,
    ];

    for pair in steps.windows(2) {
        let [lower, upper] = pair else { continue };
        assert!(lightness(*lower) < lightness(*upper), "{steps:?}");
    }
}

#[test]
fn light_cards_sit_above_the_sidebar_and_the_window() {
    let tokens = Tokens::derive(&Palette::fallback(Theme::Light), Theme::Light);

    // The sidebar is the window under the header's white scrim.
    assert!(lightness(tokens.card) > lightness(tokens.sidebar));
    assert!(lightness(tokens.sidebar) > lightness(tokens.window));
    assert!(lightness(tokens.window) > lightness(tokens.hover));
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
fn text_on_the_selection_tint_stays_readable_with_a_pale_desktop_accent() {
    let tokens = with_accent(Theme::Light, Color32::from_rgb(0xb8, 0xd8, 0xf8));

    assert!(contrast(tokens.on_tint, tokens.accent_tint) >= 4.5);
}

#[test]
fn the_light_selection_tint_stands_out_as_much_as_the_dark_one() {
    let [dark, light] = [Theme::Dark, Theme::Light].map(|theme| {
        let tokens = Tokens::derive(&Palette::fallback(theme), theme);
        contrast(tokens.accent_tint, tokens.sidebar)
    });

    assert!(light >= dark, "light {light}, dark {dark}");
}

#[test]
fn a_rim_reads_on_the_field_fill() {
    for theme in [Theme::Dark, Theme::Light] {
        let tokens = Tokens::derive(&Palette::fallback(theme), theme);

        let ratio = contrast(tokens.rim, tokens.field);
        assert!(ratio >= 3.0, "{theme:?}: {ratio}");
    }
}

pub(super) fn laid_on(over: Color32, under: Color32) -> Color32 {
    let alpha = f32::from(over.a()) / 255.0;
    let opaque = Color32::from_rgb(over.r(), over.g(), over.b())
        .gamma_multiply(1.0 / alpha.max(f32::EPSILON));

    super::mix(
        under,
        Color32::from_rgb(opaque.r(), opaque.g(), opaque.b()),
        alpha,
    )
}

#[test]
fn the_console_frost_is_as_thin_as_its_text_over_the_apps_own_surfaces_allows() {
    for theme in [Theme::Dark, Theme::Light] {
        let plain = Tokens::derive(&Palette::fallback(theme), theme);

        // Only the app's own page lies under the Console now, no desktop.
        let own = [
            plain.window,
            plain.sidebar,
            plain.card,
            plain.field,
            plain.hover,
            plain.pressed,
        ];

        for backdrop in own {
            let glass = laid_on(plain.frost, backdrop);
            let ratio = contrast(plain.text, glass);
            assert!(ratio >= 4.5, "{theme:?} over {backdrop:?}: {ratio}");
        }

        assert!(
            plain.frost.a() <= plain.frost_raised_blurred.a(),
            "{theme:?}: the Console's frost {} is thicker than a menu's {}",
            plain.frost.a(),
            plain.frost_raised_blurred.a()
        );
    }
}

#[test]
fn every_text_reads_on_the_frosted_menus_over_any_backdrop() {
    for theme in [Theme::Dark, Theme::Light] {
        let plain = Tokens::derive(&Palette::fallback(theme), theme);
        let glass = plain.frost_raised;
        assert!(glass.a() < 255, "{theme:?}: the page shows through");
        for backdrop in [Color32::WHITE, Color32::BLACK] {
            let laid = laid_on(glass, backdrop);
            let ratio = contrast(plain.secondary, laid);
            assert!(ratio >= 4.5, "{theme:?} over {backdrop:?}: {ratio}");
        }
    }
}

#[test]
fn over_the_live_blur_menus_thin_out_and_still_read_on_the_apps_own_surfaces() {
    for theme in [Theme::Dark, Theme::Light] {
        let tokens = Tokens::derive(&Palette::fallback(theme), theme);
        let own = [
            tokens.window,
            tokens.sidebar,
            tokens.card,
            tokens.raised,
            tokens.field,
            tokens.hover,
            tokens.pressed,
            super::mix(tokens.card, tokens.accent, 0.5),
        ];

        let (blurred, veil) = (tokens.frost_raised_blurred, tokens.frost_raised);

        // Light secondary text reaches 4.5:1 on the window with no room to
        // spare, so a light menu can thin only a little.
        let thinner = if theme == Theme::Dark { 40 } else { 0 };
        assert!(
            u16::from(blurred.a()) + thinner <= u16::from(veil.a()),
            "{theme:?}: {} is barely thinner than {}",
            blurred.a(),
            veil.a()
        );
        for backdrop in own {
            let ratio = contrast(tokens.secondary, laid_on(blurred, backdrop));
            assert!(ratio >= 4.5, "{theme:?} over {backdrop:?}: {ratio}");
        }
    }
}

#[test]
fn sharp_content_under_a_frosted_menu_stays_a_faint_ghost() {
    for theme in [Theme::Dark, Theme::Light] {
        let glass = Tokens::derive(&Palette::fallback(theme), theme).frost_raised;
        assert!(
            glass.a() >= 217,
            "{theme:?}: {} lets too much through",
            glass.a()
        );
    }
}

#[test]
fn the_light_blur_shows_through_as_much_as_the_dark_one() {
    for theme in [Theme::Dark, Theme::Light] {
        let tint = Tokens::derive(&Palette::fallback(theme), theme).frost_raised_blurred;
        assert!(
            tint.a() <= 191,
            "{theme:?} tint {} hides the blur",
            tint.a()
        );
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

#[test]
fn the_focus_ring_reaches_three_to_one_on_every_fill_with_any_accent() {
    for (theme, accent) in accents() {
        let tokens = with_accent(theme, accent);

        for fill in [
            tokens.window,
            tokens.sidebar,
            tokens.card,
            tokens.field,
            tokens.accent_tint,
        ] {
            let ratio = contrast(tokens.accent_ring, fill);
            assert!(
                ratio >= 3.0,
                "{theme:?} {accent:?}: ring at {ratio:.2} on {fill:?}"
            );
        }
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
            tokens.hover,
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
fn the_rim_of_a_field_reads_three_to_one_on_its_fill_and_what_it_sits_on() {
    for theme in [Theme::Dark, Theme::Light] {
        let tokens = Tokens::derive(&Palette::fallback(theme), theme);

        for fill in [tokens.field, tokens.hover, tokens.card, tokens.window] {
            let ratio = contrast(tokens.field_rim, fill);
            assert!(ratio >= 3.0, "{theme:?}: {ratio}");
        }
    }
}
