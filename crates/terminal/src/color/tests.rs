use alacritty_terminal::vte::ansi::{Color as Engine, NamedColor, Rgb};

use super::from_engine;
use crate::Color;

#[test]
fn the_sixteen_named_colours_become_palette_indices() {
    for (named, index) in [
        (NamedColor::Black, 0_u8),
        (NamedColor::Red, 1_u8),
        (NamedColor::White, 7_u8),
        (NamedColor::BrightBlack, 8_u8),
        (NamedColor::BrightBlue, 12_u8),
        (NamedColor::BrightWhite, 15_u8),
    ] {
        assert_eq!(
            from_engine(Engine::Named(named)),
            Color::Indexed(index),
            "{named:?}"
        );
    }
}

#[test]
fn dim_colours_keep_their_base_index() {
    assert_eq!(
        from_engine(Engine::Named(NamedColor::DimRed)),
        Color::Indexed(1)
    );
    assert_eq!(
        from_engine(Engine::Named(NamedColor::DimWhite)),
        Color::Indexed(7)
    );
}

#[test]
fn theme_colours_stay_unresolved() {
    for named in [
        NamedColor::Foreground,
        NamedColor::BrightForeground,
        NamedColor::DimForeground,
        NamedColor::Cursor,
    ] {
        assert_eq!(
            from_engine(Engine::Named(named)),
            Color::Foreground,
            "{named:?}"
        );
    }

    assert_eq!(
        from_engine(Engine::Named(NamedColor::Background)),
        Color::Background
    );
}

#[test]
fn palette_and_true_colours_pass_through() {
    assert_eq!(from_engine(Engine::Indexed(208)), Color::Indexed(208));
    assert_eq!(
        from_engine(Engine::Spec(Rgb { r: 1, g: 2, b: 3 })),
        Color::Rgb(1, 2, 3)
    );
}
