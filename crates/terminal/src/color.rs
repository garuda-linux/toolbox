#[cfg(test)]
mod tests;

use alacritty_terminal::vte::ansi::{self, NamedColor};

use crate::Color;

pub const fn from_engine(color: ansi::Color) -> Color {
    match color {
        ansi::Color::Named(named) => named_color(named),
        ansi::Color::Spec(rgb) => Color::Rgb(rgb.r, rgb.g, rgb.b),
        ansi::Color::Indexed(index) => Color::Indexed(index),
    }
}

/// Dim variants map to their base colour because cells carry a dim flag
/// of their own.
const fn named_color(named: NamedColor) -> Color {
    match named {
        NamedColor::Foreground
        | NamedColor::BrightForeground
        | NamedColor::DimForeground
        | NamedColor::Cursor => Color::Foreground,
        NamedColor::Background => Color::Background,
        NamedColor::Black | NamedColor::DimBlack => Color::Indexed(0),
        NamedColor::Red | NamedColor::DimRed => Color::Indexed(1),
        NamedColor::Green | NamedColor::DimGreen => Color::Indexed(2),
        NamedColor::Yellow | NamedColor::DimYellow => Color::Indexed(3),
        NamedColor::Blue | NamedColor::DimBlue => Color::Indexed(4),
        NamedColor::Magenta | NamedColor::DimMagenta => Color::Indexed(5),
        NamedColor::Cyan | NamedColor::DimCyan => Color::Indexed(6),
        NamedColor::White | NamedColor::DimWhite => Color::Indexed(7),
        NamedColor::BrightBlack => Color::Indexed(8),
        NamedColor::BrightRed => Color::Indexed(9),
        NamedColor::BrightGreen => Color::Indexed(10),
        NamedColor::BrightYellow => Color::Indexed(11),
        NamedColor::BrightBlue => Color::Indexed(12),
        NamedColor::BrightMagenta => Color::Indexed(13),
        NamedColor::BrightCyan => Color::Indexed(14),
        NamedColor::BrightWhite => Color::Indexed(15),
    }
}
