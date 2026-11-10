use super::{Settings, parse};
use crate::desktop::FontSpec;

/// `~/.config/gtk-3.0/settings.ini` of a Garuda Mokka install.
const MOKKA: &str = include_str!("../fixtures/installed/gtk-3.0-settings.ini");
/// Garuda's `/etc/skel/.config/gtk-4.0/settings.ini`.
const SKEL: &str = crate::testing::desktop::SKEL_ETC_SKEL_CONFIG_GTK_4_0_SETTINGS_INI;

/// [`SKEL`] with its font line edited to `font`.
fn skel_font(font: &str) -> Option<FontSpec> {
    let edited = SKEL.replace(
        "gtk-font-name=Fira Sans,  10",
        &format!("gtk-font-name={font}"),
    );

    assert_ne!(edited, SKEL);
    parse(&edited).font
}

#[test]
fn theme_icons_and_dark_preference_are_read() {
    let settings = parse(MOKKA);

    assert_eq!(
        settings.theme.as_deref(),
        Some("Catppuccin-Purple-Dark-Catppuccin")
    );
    assert_eq!(
        settings.icon_theme.as_deref(),
        Some("Tela-circle-dracula-dark")
    );
    assert_eq!(settings.prefer_dark, Some(true));
    assert_eq!(parse(SKEL).theme.as_deref(), Some("Sweet-Dark"));
}

#[test]
fn a_pango_font_with_a_comma_names_family_then_style() {
    assert_eq!(
        parse(MOKKA).font,
        Some(FontSpec {
            family: "Inter".to_owned(),
            points: 10.0,
            weight: 700,
        })
    );
    assert_eq!(
        parse(SKEL).font,
        Some(FontSpec {
            family: "Fira Sans".to_owned(),
            points: 10.0,
            weight: 400,
        })
    );
}

#[test]
fn a_pango_font_without_a_comma_loses_its_trailing_style_words() {
    assert_eq!(
        skel_font("Fira Sans Semi-Bold Italic 10.5"),
        Some(FontSpec {
            family: "Fira Sans".to_owned(),
            points: 10.5,
            weight: 600,
        })
    );
}

#[test]
fn a_plain_font_is_regular() {
    assert_eq!(
        skel_font("Fira Sans 10").map(|font| (font.family, font.weight)),
        Some(("Fira Sans".to_owned(), 400))
    );
}

#[test]
fn a_font_without_size_or_family_is_skipped() {
    assert_eq!(skel_font("Fira Sans,"), None);
    assert_eq!(skel_font("Bold 10"), None);
}

#[test]
fn keys_outside_the_settings_group_are_ignored() {
    assert_eq!(
        parse(&MOKKA.replace("[Settings]", "[Other]")),
        Settings::default()
    );
}

#[test]
fn gtk_enable_animations_says_whether_to_animate() {
    let animations = |value: &str| {
        let edited = MOKKA.replace(
            "gtk-enable-animations=true",
            &format!("gtk-enable-animations={value}"),
        );

        parse(&edited).enable_animations
    };

    assert_eq!(parse(MOKKA).enable_animations, Some(true));
    assert_eq!(animations("false"), Some(false));
    assert_eq!(animations("0"), Some(false));
    assert_eq!(animations("1"), Some(true));
    assert_eq!(animations("maybe"), None);
    assert_eq!(parse("[Settings]\n").enable_animations, None);
}
