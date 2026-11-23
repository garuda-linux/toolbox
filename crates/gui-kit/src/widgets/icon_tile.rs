//! A row's or a status's icon, drawn plain: a colour icon in its own
//! colours, a glyph in the text colour, on no tile as KDE draws them.

use egui::{Pos2, Rect, Ui, vec2};

use crate::icons::Icon;
use crate::tokens;

/// Kirigami's smallMedium icon, the size of a row's and a status's icon.
const SMALL_MEDIUM: f32 = 22.0;

/// An icon's box: beside a row's title and detail, or beside one line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileSize {
    Row,
    Line,
}

impl TileSize {
    /// The icon's edge in points. The desktop's icon sizes do not follow
    /// the body font.
    pub const fn edge(self) -> f32 {
        match self {
            Self::Row | Self::Line => SMALL_MEDIUM,
        }
    }
}

pub fn paint_icon(ui: &Ui, centre: Pos2, size: TileSize, icon: Icon) {
    let edge = size.edge();
    let tint = tokens::current(ui.ctx()).text;
    let theme = crate::icons::theme_in_use(ui.ctx());
    icon.image(theme.as_deref(), tint)
        .fit_to_exact_size(vec2(edge, edge))
        .paint_at(ui, Rect::from_center_size(centre, vec2(edge, edge)));
}
