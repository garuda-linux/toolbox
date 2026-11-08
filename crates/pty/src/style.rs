/// A colour as the program asked for it. Palette entries stay indices so
/// the theme decides what "red" looks like.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    Foreground,
    /// Separate from Foreground, as inverse video swaps the two.
    Background,
    /// An entry of the 256 colour palette. 0 to 15 are the ANSI colours,
    /// above them xterm's cube and gray ramp.
    Indexed(u8),
    Rgb(u8, u8, u8),
}

/// How a run of text looks. Inverse video is already applied to the
/// colours, so the UI never swaps them itself.
#[expect(
    clippy::struct_excessive_bools,
    reason = "each flag is an independent text attribute the UI reads"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    /// Slanted text (SGR 3).
    pub italic: bool,
    /// Any kind of underline (single, double, curly, dotted, dashed).
    pub underline: bool,
    /// Faint text (SGR 2).
    pub dim: bool,
    pub strikeout: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fg: Color::Foreground,
            bg: Color::Background,
            bold: false,
            italic: false,
            underline: false,
            dim: false,
            strikeout: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Run {
    /// The characters, combining marks included.
    pub text: String,
    /// How many cells the run covers. More than the character count when
    /// it holds wide (CJK) characters, which take two cells each.
    pub columns: usize,
    pub style: Style,
}
