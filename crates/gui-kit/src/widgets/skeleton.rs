//! Placeholders shaped like the rows a page is loading, in place of a
//! spinner (spotifast's skeletons). A light band sweeps across them, so
//! a long wait still shows that something works. With less motion asked
//! for, or out of sight, the bars stand still and ask for no frames.

mod shine;

use egui::{
    Color32, Context, Id, Painter, Rect, Response, Role, Sense, Stroke, Ui, Vec2, WidgetInfo, pos2,
    vec2,
};
use shine::{band_middle, paint_shine};

use super::ICON_EDGE;
use super::card::DIVIDER;
use crate::motion::{self, Timing};
use crate::theme::TILE_RADIUS;
use crate::tokens;

/// The share of its room each bar takes, varied by row so the
/// placeholder reads as a list rather than a grid.
const SHARES: [f32; 5] = [0.62, 0.48, 0.71, 0.55, 0.4];
/// Bars never grow past this, as no title is this long.
const MAX_BAR: f32 = 420.0;
const TITLE_BAR: f32 = 12.0;
const DETAIL_BAR: f32 = 9.0;
/// Half the gap between the title and the detail bar, which meet at the
/// row's middle.
const HALF_BAR_GAP: f32 = 3.0;
const CONTROL: Vec2 = super::switch::SIZE;
const BAR_RADIUS: f32 = 4.0;

/// The rows a [`skeleton_list`] stands in for.
#[derive(Clone, Copy, Debug)]
pub struct SkeletonRows {
    pub count: usize,
    pub row_height: f32,
    /// A tile before each row's text, for rows that start with an icon,
    /// so the text does not move when the rows come.
    pub leading: bool,
}

/// What every placeholder row of a list shares.
#[derive(Clone, Copy)]
struct RowLook {
    fill: Color32,
    /// The band's light and where its middle is, while it sweeps.
    band: Option<(Color32, f32)>,
    /// From the row's ends to its parts.
    padding: f32,
    /// The leading tile's edge, if the rows have one.
    tile: Option<f32>,
    /// From the row's padding to its bars.
    indent: f32,
    /// The longest a bar may be.
    room: f32,
}

/// `count` rows of `row_height` without a tile, see [`skeleton_list`].
pub fn skeleton_rows(ui: &mut Ui, count: usize, row_height: f32, name: &str) -> Response {
    let rows = SkeletonRows {
        count,
        row_height,
        leading: false,
    };

    skeleton_list(ui, rows, name)
}

/// `rows`, each with a title bar, a description bar and a control at the
/// right, a divider between them as a card's rows have. Screen readers
/// hear `name` once.
pub fn skeleton_list(ui: &mut Ui, rows: SkeletonRows, name: &str) -> Response {
    let SkeletonRows {
        count,
        row_height,
        leading,
    } = rows;

    // Every row but the first has its divider's pixel above it, as the
    // rows that come do.
    let step = row_height + DIVIDER;
    let height = step.mul_add(points(count), -DIVIDER).max(0.0);
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());

    response.widget_info(|| WidgetInfo::labeled(Role::ProgressIndicator, true, name));

    let tokens = tokens::current(ui.ctx());
    let padding = tokens.sizes.row_padding;
    let painter = ui.painter_at(rect);
    let tile = leading.then_some(ICON_EDGE);
    let indent = tile.map_or(0.0, |edge| edge + tokens.sizes.picture_gap);
    let room = (padding.x.mul_add(-3.0, rect.width()) - CONTROL.x - indent).clamp(0.0, MAX_BAR);
    let look = RowLook {
        fill: tokens.button_hover,
        band: band_middle(ui, rect).map(|middle| (tokens.shimmer, middle)),
        padding: padding.x,
        tile,
        indent,
        room,
    };

    for index in 0..count {
        let top = step.mul_add(points(index), rect.top());
        let row = Rect::from_min_size(pos2(rect.left(), top), vec2(rect.width(), row_height));
        if index > 0 {
            let line = painter.round_to_pixel_center(top - DIVIDER / 2.0);
            let ends = (rect.left() + padding.x)..=(rect.right() - padding.x);
            painter.hline(ends, line, Stroke::new(DIVIDER, tokens.outline));
        }

        if ui.is_rect_visible(row) {
            paint_row(&painter, row, index, look);
        }
    }

    response
}

fn paint_row(painter: &Painter, row: Rect, index: usize, look: RowLook) {
    let middle = row.center().y;
    let bar = |rect: Rect, radius: f32| {
        painter.rect_filled(rect, radius, look.fill);
        if let Some((light, at)) = look.band {
            paint_shine(painter, rect, radius, at, light);
        }
    };

    if let Some(edge) = look.tile {
        let at = pos2(row.left() + look.padding, middle - edge / 2.0);
        bar(
            Rect::from_min_size(at, vec2(edge, edge)),
            f32::from(TILE_RADIUS),
        );
    }

    let left = row.left() + look.padding + look.indent;
    let title = Rect::from_min_size(
        pos2(left, middle - TITLE_BAR - HALF_BAR_GAP),
        vec2(look.room * share(index, 0), TITLE_BAR),
    );

    let detail = Rect::from_min_size(
        pos2(left, middle + HALF_BAR_GAP),
        vec2(look.room * share(index, 1), DETAIL_BAR),
    );

    let control = Rect::from_center_size(
        pos2(row.right() - look.padding - CONTROL.x / 2.0, middle),
        CONTROL,
    );

    bar(title, BAR_RADIUS);
    bar(detail, BAR_RADIUS);
    bar(control, CONTROL.y / 2.0);
}

fn share(index: usize, bar: usize) -> f32 {
    let base = SHARES
        .get((index * 2 + bar) % SHARES.len())
        .copied()
        .unwrap_or(0.5);

    if bar == 0 { base } else { base * 0.7 }
}

/// No page shows more rows than a `u16` holds.
fn points(count: usize) -> f32 {
    f32::from(u16::try_from(count).unwrap_or(u16::MAX))
}

/// Notes that a skeleton stands in for the content of `id` this frame,
/// so the content fades in when it comes, as [`after_skeleton`] draws it.
pub fn hold_content(ctx: &Context, id: Id) {
    motion::settle(ctx, id, false);
}

/// The content of `id`. It fades in the first time it takes the place of
/// a skeleton that [`hold_content`] noted, and draws at once ever after,
/// so a refresh or a search never fades.
pub fn after_skeleton<R>(ui: &mut Ui, id: Id, body: impl FnOnce(&mut Ui) -> R) -> R {
    let timing = Timing::new(motion::CONTENT_FADE, 0.0);
    let shown = motion::presence(ui.ctx(), id, true, timing);
    ui.scope(|ui| {
        if shown < 1.0 {
            ui.multiply_opacity(shown);
        }

        body(ui)
    })
    .inner
}
