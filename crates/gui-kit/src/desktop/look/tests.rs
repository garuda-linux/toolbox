#![expect(
    clippy::disallowed_methods,
    reason = "tests compare against literal colours"
)]

use egui::{Color32, Theme};

use super::{Look, Portal, Sources};
use crate::Palette;
use crate::desktop::{Colors, DesktopTheme, FontSpec};

const BLUE: Color32 = Color32::from_rgb(0x35, 0x84, 0xe4);

fn light_desktop() -> DesktopTheme {
    DesktopTheme {
        colors: Colors {
            window: Some(Color32::from_rgb(0xef, 0xf0, 0xf1)),
            text: Some(Color32::from_rgb(0x23, 0x26, 0x29)),
            ..Colors::default()
        },
        font: Some(FontSpec {
            family: "Noto Sans".to_owned(),
            points: 10.0,
            weight: 400,
        }),
        fixed_font: None,
        icon_theme: Some("breeze".to_owned()),
        animations: None,
    }
}

#[test]
fn without_any_source_the_fallback_is_dark() {
    let look = Look::resolve(&Sources::default());

    assert_eq!(look.theme, Theme::Dark);
    assert_eq!(look.dark, Palette::fallback(Theme::Dark));
    assert_eq!(look.light, Palette::fallback(Theme::Light));
}

#[test]
fn the_portal_scheme_picks_the_fallback_theme() {
    let look = Look::resolve(&Sources {
        portal: Portal {
            dark: Some(false),
            accent: None,
            animations: None,
        },
        ..Sources::default()
    });

    assert_eq!(look.theme, Theme::Light);
}

#[test]
fn a_desktop_window_colour_decides_the_theme_over_the_portal() {
    let look = Look::resolve(&Sources {
        portal: Portal {
            dark: Some(true),
            accent: None,
            animations: None,
        },
        desktop: Some(light_desktop()),
        prefer_dark: None,
    });

    assert_eq!(look.theme, Theme::Light);
    assert_eq!(look.light.window, Color32::from_rgb(0xef, 0xf0, 0xf1));
    assert_eq!(look.dark, Palette::fallback(Theme::Dark));
}

#[test]
fn gtk_prefer_dark_counts_when_the_portal_says_nothing() {
    let look = Look::resolve(&Sources {
        prefer_dark: Some(false),
        ..Sources::default()
    });

    assert_eq!(look.theme, Theme::Light);
}

#[test]
fn the_portal_accent_fills_in_when_the_desktop_names_none() {
    let look = Look::resolve(&Sources {
        portal: Portal {
            dark: None,
            accent: Some(BLUE),
            animations: None,
        },
        desktop: Some(light_desktop()),
        prefer_dark: None,
    });

    assert_eq!(look.light.accent, BLUE);
    assert_eq!(look.light.on_accent, Color32::WHITE);
}

#[test]
fn the_desktop_accent_wins_over_the_portal() {
    let mut desktop = light_desktop();
    desktop.colors.accent = Some(Color32::from_rgb(0x3d, 0xae, 0xe9));
    let look = Look::resolve(&Sources {
        portal: Portal {
            dark: None,
            accent: Some(BLUE),
            animations: None,
        },
        desktop: Some(desktop),
        prefer_dark: None,
    });

    assert_eq!(look.light.accent, Color32::from_rgb(0x3d, 0xae, 0xe9));
}

#[test]
fn a_light_portal_accent_gets_dark_text() {
    let look = Look::resolve(&Sources {
        portal: Portal {
            dark: Some(true),
            accent: Some(Color32::from_rgb(0xf9, 0xe2, 0xaf)),
            animations: None,
        },
        ..Sources::default()
    });

    assert_eq!(look.dark.on_accent, Color32::BLACK);
}

#[test]
fn a_portal_accent_in_unit_range_becomes_a_colour() {
    assert_eq!(
        Portal::accent_from_unit_rgb(1.0, 0.5, 0.0),
        Some(Color32::from_rgb(255, 128, 0))
    );
}

#[test]
fn a_portal_accent_out_of_range_means_unset() {
    assert_eq!(Portal::accent_from_unit_rgb(-1.0, 0.5, 0.0), None);
    assert_eq!(Portal::accent_from_unit_rgb(0.5, 1.5, 0.0), None);
    assert_eq!(Portal::accent_from_unit_rgb(f64::NAN, 0.5, 0.0), None);
}

#[test]
fn fonts_and_icon_theme_come_from_the_desktop() {
    let look = Look::resolve(&Sources {
        desktop: Some(light_desktop()),
        ..Sources::default()
    });

    assert_eq!(
        look.font.map(|font| font.family),
        Some("Noto Sans".to_owned())
    );
    assert_eq!(look.icon_theme.as_deref(), Some("breeze"));
}

#[test]
fn motion_is_full_while_no_source_turns_animations_off() {
    let on = Look::resolve(&Sources {
        portal: Portal {
            animations: Some(true),
            ..Portal::default()
        },
        ..Sources::default()
    });

    assert!(!Look::resolve(&Sources::default()).reduced_motion);
    assert!(!on.reduced_motion);
}

#[test]
fn the_portal_turning_animations_off_reduces_motion() {
    let look = Look::resolve(&Sources {
        portal: Portal {
            animations: Some(false),
            ..Portal::default()
        },
        ..Sources::default()
    });

    assert!(look.reduced_motion);
}

#[test]
fn the_desktop_files_turning_animations_off_reduce_motion_over_the_portal() {
    let look = Look::resolve(&Sources {
        portal: Portal {
            animations: Some(true),
            ..Portal::default()
        },
        desktop: Some(DesktopTheme {
            animations: Some(false),
            ..light_desktop()
        }),
        ..Sources::default()
    });

    assert!(look.reduced_motion);
}
