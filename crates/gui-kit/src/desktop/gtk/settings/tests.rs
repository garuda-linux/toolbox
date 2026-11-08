use super::{Settings, parse};
use crate::desktop::FontSpec;

const CATPPUCCIN: &str = "\
[Settings]
gtk-application-prefer-dark-theme=true
gtk-font-name=Inter, Bold 10
gtk-icon-theme-name=Tela-circle-dark
gtk-theme-name=catppuccin-mocha-mauve-standard+default
";

#[test]
fn theme_icons_and_dark_preference_are_read() {
    let settings = parse(CATPPUCCIN);

    assert_eq!(
        settings.theme.as_deref(),
        Some("catppuccin-mocha-mauve-standard+default")
    );
    assert_eq!(settings.icon_theme.as_deref(), Some("Tela-circle-dark"));
    assert_eq!(settings.prefer_dark, Some(true));
}

#[test]
fn a_pango_font_with_a_comma_names_family_then_style() {
    assert_eq!(
        parse(CATPPUCCIN).font,
        Some(FontSpec {
            family: "Inter".to_owned(),
            points: 10.0,
            weight: 700,
        })
    );
}

#[test]
fn a_pango_font_without_a_comma_loses_its_trailing_style_words() {
    let font = parse("[Settings]\ngtk-font-name=Noto Sans Semi-Bold Italic 10.5\n").font;

    assert_eq!(
        font,
        Some(FontSpec {
            family: "Noto Sans".to_owned(),
            points: 10.5,
            weight: 600,
        })
    );
}

#[test]
fn a_plain_font_is_regular() {
    let font = parse("[Settings]\ngtk-font-name=Cantarell 11\n").font;

    assert_eq!(
        font.map(|font| (font.family, font.weight)),
        Some(("Cantarell".to_owned(), 400))
    );
}

#[test]
fn a_font_without_size_or_family_is_skipped() {
    assert_eq!(parse("[Settings]\ngtk-font-name=Cantarell\n").font, None);
    assert_eq!(parse("[Settings]\ngtk-font-name=Bold 10\n").font, None);
}

#[test]
fn keys_outside_the_settings_group_are_ignored() {
    assert_eq!(
        parse("[Other]\ngtk-theme-name=Adwaita\n"),
        Settings::default()
    );
}

#[test]
fn gtk_enable_animations_says_whether_to_animate() {
    let animations = |value: &str| {
        parse(&format!("[Settings]\ngtk-enable-animations={value}\n")).enable_animations
    };

    assert_eq!(animations("false"), Some(false));
    assert_eq!(animations("0"), Some(false));
    assert_eq!(animations("true"), Some(true));
    assert_eq!(animations("1"), Some(true));
    assert_eq!(animations("maybe"), None);
    assert_eq!(parse("[Settings]\n").enable_animations, None);
}
