use std::sync::Arc;

use egui::{TextStyle, Theme};
use toolbox_gui::Setup;
use toolbox_gui::desktop::DesktopLook;
use toolbox_gui_kit::Palette;
use toolbox_gui_kit::desktop::FontSpec;
use toolbox_gui_kit::icons::{ThemeIcons, theme_in_use};
use toolbox_gui_kit::tokens::Tokens;

use crate::common::launch;

#[test]
fn first_frame_uses_the_fallback_palette() {
    let harness = launch(Setup::default());

    let fill = harness.ctx.style_of(Theme::Dark).visuals.panel_fill;
    let tokens = Tokens::derive(&Palette::fallback(Theme::Dark), Theme::Dark);
    assert_eq!(fill, tokens.window);
}

#[test]
fn a_new_desktop_look_takes_over_theme_and_colours() {
    let (sender, receiver) = tokio::sync::watch::channel(DesktopLook::default());
    let mut harness = launch(Setup {
        look: Some(receiver),
        ..Setup::default()
    });

    let mut light = DesktopLook::default();
    light.look.theme = Theme::Light;
    light.look.light.window = Palette::fallback(Theme::Dark).accent;
    sender.send(light.clone()).unwrap();
    harness.run();

    assert_eq!(harness.ctx.theme(), Theme::Light);
    assert_eq!(
        harness.ctx.style_of(Theme::Light).visuals.panel_fill,
        Tokens::derive(&light.look.light, Theme::Light).window
    );
}

#[test]
fn the_desktop_icon_theme_reaches_the_icon_widgets() {
    let (sender, receiver) = tokio::sync::watch::channel(DesktopLook::default());
    let mut harness = launch(Setup {
        look: Some(receiver),
        ..Setup::default()
    });

    let icons = Arc::new(ThemeIcons::new("Papirus"));
    sender
        .send(DesktopLook {
            icons: Some(Arc::clone(&icons)),
            ..DesktopLook::default()
        })
        .unwrap();
    harness.run();

    assert_eq!(theme_in_use(&harness.ctx), Some(icons));
}

#[test]
fn the_desktop_font_size_sets_the_text_size() {
    let (sender, receiver) = tokio::sync::watch::channel(DesktopLook::default());
    let mut harness = launch(Setup {
        look: Some(receiver),
        ..Setup::default()
    });

    let mut larger = DesktopLook::default();
    larger.look.font = Some(FontSpec {
        family: "Not Installed".to_owned(),
        points: 12.0,
        weight: 400,
    });
    sender.send(larger).unwrap();
    harness.run();

    let body = harness.ctx.global_style().text_styles[&TextStyle::Body].size;
    assert!((body - 16.0).abs() < 0.01, "body text is {body} points");
}
