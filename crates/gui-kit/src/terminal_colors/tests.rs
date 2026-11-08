#![expect(
    clippy::disallowed_methods,
    reason = "tests compare against literal colours"
)]

use egui::{Color32, Theme};
use toolbox_terminal::Color;

use super::{Ansi, resolve};
use crate::Palette;

fn dark() -> (Palette, Ansi) {
    let palette = Palette::fallback(Theme::Dark);
    (palette, Ansi::of(&palette, true))
}

#[test]
fn foreground_and_background_are_the_palettes_text_and_view() {
    let (palette, ansi) = dark();

    assert_eq!(resolve(Color::Foreground, &ansi, &palette), palette.text);
    assert_eq!(resolve(Color::Background, &ansi, &palette), palette.view);
}

#[test]
fn the_status_colours_name_red_green_and_yellow() {
    let (palette, ansi) = dark();

    assert_eq!(resolve(Color::Indexed(1), &ansi, &palette), palette.error);
    assert_eq!(resolve(Color::Indexed(2), &ansi, &palette), palette.success);
    assert_eq!(resolve(Color::Indexed(3), &ansi, &palette), palette.warning);
    assert_eq!(resolve(Color::Indexed(4), &ansi, &palette), palette.accent);
}

#[test]
fn bright_colours_are_lighter_on_dark_and_darker_on_light() {
    let (palette, ansi) = dark();
    let red = resolve(Color::Indexed(1), &ansi, &palette);
    let bright_red = resolve(Color::Indexed(9), &ansi, &palette);
    assert!(bright_red.intensity() > red.intensity());

    let light = Palette::fallback(Theme::Light);
    let light_ansi = Ansi::of(&light, false);
    assert!(
        resolve(Color::Indexed(9), &light_ansi, &light).intensity()
            < resolve(Color::Indexed(1), &light_ansi, &light).intensity()
    );
}

#[test]
fn the_cube_and_gray_ramp_follow_xterm() {
    let (palette, ansi) = dark();

    assert_eq!(resolve(Color::Indexed(16), &ansi, &palette), Color32::BLACK);
    assert_eq!(
        resolve(Color::Indexed(196), &ansi, &palette),
        Color32::from_rgb(255, 0, 0)
    );
    assert_eq!(
        resolve(Color::Indexed(232), &ansi, &palette),
        Color32::from_rgb(8, 8, 8)
    );
    assert_eq!(
        resolve(Color::Indexed(255), &ansi, &palette),
        Color32::from_rgb(238, 238, 238)
    );
}

#[test]
fn true_colours_pass_through() {
    let (palette, ansi) = dark();

    assert_eq!(
        resolve(Color::Rgb(1, 2, 3), &ansi, &palette),
        Color32::from_rgb(1, 2, 3)
    );
}

#[test]
fn a_palette_with_terminal_colours_of_its_own_keeps_them() {
    let own: [Color32; 16] =
        std::array::from_fn(|index| Color32::from_rgb(u8::try_from(index).unwrap(), 0x40, 0x80));

    let palette = Palette {
        ansi: Some(Ansi::new(own)),
        ..Palette::fallback(Theme::Dark)
    };

    for dark in [true, false] {
        let ansi = Ansi::of(&palette, dark);
        for (index, colour) in own.iter().enumerate() {
            let index = u8::try_from(index).unwrap();
            assert_eq!(resolve(Color::Indexed(index), &ansi, &palette), *colour);
        }
    }
}
