#![expect(
    clippy::disallowed_methods,
    reason = "tests compare against literal colours"
)]

use egui::Color32;

use super::parse;
use crate::desktop::FontSpec;

const CATPPUCCIN: &str = "\
[ColorEffects:Disabled]
Color=56,56,56

[Colors:Button]
BackgroundNormal=49, 50, 68
ForegroundNormal=205, 214, 244

[Colors:Selection]
BackgroundNormal=203,166,247
ForegroundNormal=17, 17, 27

[Colors:View]
BackgroundNormal=30, 30, 46
ForegroundNegative=243, 139, 168
ForegroundNeutral=249, 226, 175
ForegroundPositive=166, 227, 161

[Colors:Window]
BackgroundNormal=24, 24, 37
ForegroundInactive=166, 173, 200
ForegroundNormal=205, 214, 244

[Colors:Window][Inactive]
BackgroundNormal=1, 2, 3

[General]
fixed=JetBrainsMonoNL Nerd Font,10,-1,5,400,0,0,0,0,0,0,0,0,0,0,1
font=Inter,10.5,-1,5,700,0,0,0,0,0,0,0,0,0,0,1,Bold
Name[de]=Irgendwas

[Icons]
Theme=Tela-circle-dark
";

#[test]
fn colours_come_from_their_kde_roles() {
    let colors = parse(CATPPUCCIN).colors;
    assert_eq!(colors.window, Some(Color32::from_rgb(24, 24, 37)));
    assert_eq!(colors.view, Some(Color32::from_rgb(30, 30, 46)));
    assert_eq!(colors.text, Some(Color32::from_rgb(205, 214, 244)));
    assert_eq!(colors.accent, Some(Color32::from_rgb(203, 166, 247)));
    assert_eq!(colors.on_accent, Some(Color32::from_rgb(17, 17, 27)));
    assert_eq!(colors.error, Some(Color32::from_rgb(243, 139, 168)));
    assert_eq!(colors.warning, Some(Color32::from_rgb(249, 226, 175)));
    assert_eq!(colors.success, Some(Color32::from_rgb(166, 227, 161)));
}

#[test]
fn link_and_visited_colours_are_read() {
    let colors = parse(
        "[Colors:View]\nForegroundLink=29,153,243\nForegroundVisited=155,89,182\n\
         [Colors:Window]\nForegroundLink=1,2,3\n",
    )
    .colors;

    assert_eq!(colors.link, Some(Color32::from_rgb(29, 153, 243)));
    assert_eq!(colors.visited, Some(Color32::from_rgb(155, 89, 182)));
}

#[test]
fn a_scheme_without_link_colours_names_none() {
    let colors = parse(CATPPUCCIN).colors;

    assert_eq!(colors.link, None);
    assert_eq!(colors.visited, None);
}

#[test]
fn a_state_section_does_not_override_the_normal_one() {
    let colors = parse(CATPPUCCIN).colors;
    assert_eq!(colors.window, Some(Color32::from_rgb(24, 24, 37)));
}

#[test]
fn fonts_and_icon_theme_are_read() {
    let theme = parse(CATPPUCCIN);
    assert_eq!(
        theme.font,
        Some(FontSpec {
            family: "Inter".to_owned(),
            points: 10.5,
            weight: 700,
        })
    );
    assert_eq!(
        theme.fixed_font.map(|font| font.family),
        Some("JetBrainsMonoNL Nerd Font".to_owned())
    );
    assert_eq!(theme.icon_theme.as_deref(), Some("Tela-circle-dark"));
}

#[test]
fn qt5_font_weights_become_css_weights() {
    let theme = parse("[General]\nfont=Noto Sans,10,-1,5,50,0,0,0,0,0\n");
    assert_eq!(theme.font.map(|font| font.weight), Some(400));

    let bold = parse("[General]\nfont=Noto Sans,10,-1,5,75,0,0,0,0,0\n");
    assert_eq!(bold.font.map(|font| font.weight), Some(700));
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
    let off = parse("[KDE]\nAnimationDurationFactor=0\n");
    let slow = parse("[KDE]\nAnimationDurationFactor=2.5\n");

    assert_eq!(off.animations, Some(false));
    assert_eq!(slow.animations, Some(true));
    assert_eq!(parse("").animations, None);
    assert_eq!(
        parse("[KDE]\nAnimationDurationFactor=fast\n").animations,
        None
    );
    assert_eq!(
        parse("[General]\nAnimationDurationFactor=0\n").animations,
        None
    );
}
