//! The tinted rounded square behind a row's or a section's icon.
//!
//! Its hue is the open page's ([`tokens::page_hue`]), so a page's icons
//! share one colour and a row never picks its own.

use egui::{CornerRadius, Rect, Ui, vec2};

use crate::icons::Icon;
use crate::theme::{LARGE_TILE_RADIUS, SMALL_TILE_RADIUS, TILE_RADIUS};
use crate::tokens::{self, Tile};
use crate::ui_ext::icon_image;

const GLYPH_SHARE: f32 = 0.56;

/// A tile's size: beside a row's title and detail, beside one line, or
/// alone above an empty state's title.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileSize {
    Row,
    Line,
    Large,
}

impl TileSize {
    /// The tile's edge in points.
    pub fn edge(self, ui: &Ui) -> f32 {
        let sizes = tokens::current(ui.ctx()).sizes;
        match self {
            Self::Row => sizes.icon_tile,
            Self::Line => sizes.icon_tile_small,
            Self::Large => sizes.icon_tile_large,
        }
    }

    const fn radius(self) -> u8 {
        match self {
            Self::Row => TILE_RADIUS,
            Self::Line => SMALL_TILE_RADIUS,
            Self::Large => LARGE_TILE_RADIUS,
        }
    }
}

pub fn page_tile(ctx: &egui::Context) -> Tile {
    tokens::current(ctx).tiles.of(tokens::page_hue(ctx))
}

pub fn paint_icon_tile(ui: &Ui, centre: egui::Pos2, size: TileSize, icon: Icon, tile: Tile) {
    let edge = size.edge(ui);
    let square = Rect::from_center_size(centre, vec2(edge, edge));
    ui.painter()
        .rect_filled(square, CornerRadius::same(size.radius()), tile.fill);

    let glyph = edge * GLYPH_SHARE;
    icon_image(ui, icon, tile.ink).paint_at(ui, Rect::from_center_size(centre, vec2(glyph, glyph)));
}

pub fn icon_tile(ui: &mut Ui, size: TileSize, icon: Icon) -> Rect {
    let edge = size.edge(ui);
    let (rect, _) = ui.allocate_exact_size(vec2(edge, edge), egui::Sense::hover());
    paint_icon_tile(ui, rect.center(), size, icon, page_tile(ui.ctx()));
    rect
}
