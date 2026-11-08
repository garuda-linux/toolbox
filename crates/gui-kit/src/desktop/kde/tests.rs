#![expect(
    clippy::disallowed_methods,
    reason = "tests compare against literal colours"
)]

use egui::Color32;

use super::parse;
use crate::desktop::FontSpec;

/// `~/.config/kdeglobals` of a Garuda Mokka install: Catppuccin colours,
/// Qt 5 font strings.
const MOKKA: &str = include_str!("fixtures/installed/kdeglobals");
/// `/usr/share/color-schemes/BreezeDark.colors`.
const BREEZE_DARK: &str = include_str!("fixtures/BreezeDark.colors");
/// Garuda's `/etc/skel/.config/kdeglobals`: fonts in the Qt 6 form and
/// no colours.
const SKEL: &str = include_str!("../fixtures/skel/etc/skel/.config/kdeglobals");
/// `~/.config/kdeglobals` after Plasma's setting for no animations.
const NO_ANIMATIONS: &str = include_str!("fixtures/kdeglobals-written");

#[test]
fn colours_come_from_their_kde_roles() {
    let colors = parse(MOKKA).colors;
    assert_eq!(colors.window, Some(Color32::from_rgb(30, 30, 46)));
    assert_eq!(colors.view, Some(Color32::from_rgb(30, 30, 46)));
    assert_eq!(colors.text, Some(Color32::from_rgb(205, 214, 244)));
    assert_eq!(colors.accent, Some(Color32::from_rgb(203, 166, 247)));
    assert_eq!(colors.on_accent, Some(Color32::from_rgb(30, 30, 46)));
    assert_eq!(colors.error, Some(Color32::from_rgb(243, 139, 168)));
    assert_eq!(colors.warning, Some(Color32::from_rgb(249, 226, 175)));
    assert_eq!(colors.success, Some(Color32::from_rgb(166, 227, 161)));

    let breeze = parse(BREEZE_DARK).colors;
    assert_eq!(breeze.window, Some(Color32::from_rgb(32, 35, 38)));
    assert_eq!(breeze.view, Some(Color32::from_rgb(20, 22, 24)));
    assert_eq!(breeze.accent, Some(Color32::from_rgb(61, 174, 233)));
}

#[test]
fn link_and_visited_colours_are_read() {
    let colors = parse(BREEZE_DARK).colors;

    assert_eq!(colors.link, Some(Color32::from_rgb(29, 153, 243)));
    assert_eq!(colors.visited, Some(Color32::from_rgb(155, 89, 182)));
}

#[test]
fn a_file_without_link_colours_names_none() {
    let colors = parse(SKEL).colors;

    assert_eq!(colors.link, None);
    assert_eq!(colors.visited, None);
}

#[test]
fn a_state_section_does_not_override_the_normal_one() {
    // Breeze Dark's inactive header section edited into one of the
    // window, with a colour of its own.
    let edited = BREEZE_DARK.replace(
        "[Colors:Header][Inactive]\nBackgroundAlternate=41,44,48\nBackgroundNormal=32,35,38",
        "[Colors:Window][Inactive]\nBackgroundAlternate=41,44,48\nBackgroundNormal=1,2,3",
    );

    assert_ne!(edited, BREEZE_DARK);
    assert_eq!(
        parse(&edited).colors.window,
        Some(Color32::from_rgb(32, 35, 38))
    );
}

#[test]
fn fonts_and_icon_theme_are_read() {
    // No capture names an icon theme, so one is added.
    let theme = parse(&format!("{MOKKA}\n[Icons]\nTheme=breeze-dark\n"));
    assert_eq!(
        theme.font,
        Some(FontSpec {
            family: "Inter".to_owned(),
            points: 10.0,
            weight: 700,
        })
    );
    assert_eq!(
        theme.fixed_font.map(|font| font.family),
        Some("Liberation Mono".to_owned())
    );
    assert_eq!(theme.icon_theme.as_deref(), Some("breeze-dark"));
    assert_eq!(parse(MOKKA).icon_theme, None);
}

#[test]
fn qt5_font_weights_become_css_weights() {
    let skel = parse(SKEL);
    assert_eq!(
        skel.font,
        Some(FontSpec {
            family: "Fira Sans".to_owned(),
            points: 10.0,
            weight: 400,
        })
    );

    // Mokka writes the Qt 5 weight 75 for bold.
    assert_eq!(parse(MOKKA).font.map(|font| font.weight), Some(700));

    // Mokka's font edited to the Qt 5 weight of a regular face.
    let regular = MOKKA.replace(
        "font=Inter,10,-1,5,75,0,0,0,0,0,Bold",
        "font=Inter,10,-1,5,50,0,0,0,0,0",
    );

    assert_eq!(parse(&regular).font.map(|font| font.weight), Some(400));
}

#[test]
fn broken_values_are_skipped_not_guessed() {
    let theme = parse(
        "[Colors:Window]\nBackgroundNormal=300,0,0\nForegroundNormal=1,2\n\
         [General]\nfont=,10\nfixed=Mono,big\n",
    );

    assert_eq!(theme.colors.window, None);
    assert_eq!(theme.colors.text, None);
    assert_eq!(theme.font, None);
    assert_eq!(theme.fixed_font, None);
}

#[test]
fn an_empty_file_overrides_nothing() {
    let theme = parse("");
    assert_eq!(theme.colors, crate::desktop::Colors::default());
    assert_eq!(theme.icon_theme, None);
}

#[test]
fn an_animation_speed_of_zero_turns_animations_off() {
    // The written file edited to a slower speed, a word and another
    // section.
    let slow = NO_ANIMATIONS.replace("Factor=0", "Factor=2.5");
    let word = NO_ANIMATIONS.replace("Factor=0", "Factor=fast");
    let elsewhere = NO_ANIMATIONS.replace("[KDE]", "[General]");

    assert_eq!(parse(NO_ANIMATIONS).animations, Some(false));
    assert_eq!(parse(&slow).animations, Some(true));
    assert_eq!(parse(MOKKA).animations, None);
    assert_eq!(parse(&word).animations, None);
    assert_eq!(parse(&elsewhere).animations, None);
}
