//! Placeholders shaped like the rows a page is loading, in place of a
//! spinner (spotifast's skeletons). They are drawn once and do not
//! move, so a wait asks for no frames.

use egui::{
    Context, CornerRadius, Id, Rect, Response, Role, Sense, Ui, Vec2, WidgetInfo, pos2, vec2,
};

use crate::motion::{self, Timing};
use crate::tokens;

/// The share of its room each bar takes, varied by row so the
/// placeholder reads as a list rather than a grid.
const SHARES: [f32; 5] = [0.62, 0.48, 0.71, 0.55, 0.4];
/// Bars never grow past this, as no title is this long.
const MAX_BAR: f32 = 420.0;
const TITLE_BAR: f32 = 12.0;
const DETAIL_BAR: f32 = 9.0;
const CONTROL: Vec2 = super::switch::SIZE;
const BAR_RADIUS: u8 = 4;

/// `count` rows of `row_height`, each with a title bar, a description bar
/// and a control at the right. Screen readers hear `name` once.
pub fn skeleton_rows(ui: &mut Ui, count: usize, row_height: f32, name: &str) -> Response {
    let height = row_height * points(count);
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());

    response.widget_info(|| WidgetInfo::labeled(Role::ProgressIndicator, true, name));

    let tokens = tokens::current(ui.ctx());
    let padding = tokens.sizes.row_padding;
    let fill = tokens.hover;
    let painter = ui.painter_at(rect);
    let room = (padding.x.mul_add(-3.0, rect.width()) - CONTROL.x).clamp(0.0, MAX_BAR);

    for index in 0..count {
        let top = row_height.mul_add(points(index), rect.top());
        let row = Rect::from_min_size(pos2(rect.left(), top), vec2(rect.width(), row_height));
        if !ui.is_rect_visible(row) {
            continue;
        }

        let middle = row.center().y;
        let left = row.left() + padding.x;
        let title = Rect::from_min_size(
            pos2(left, middle - TITLE_BAR - 3.0),
            vec2(room * share(index, 0), TITLE_BAR),
        );

        let detail = Rect::from_min_size(
            pos2(left, middle + 3.0),
            vec2(room * share(index, 1), DETAIL_BAR),
        );

        let control = Rect::from_center_size(
            pos2(row.right() - padding.x - CONTROL.x / 2.0, middle),
            CONTROL,
        );

        painter.rect_filled(title, CornerRadius::same(BAR_RADIUS), fill);
        painter.rect_filled(detail, CornerRadius::same(BAR_RADIUS), fill);
        painter.rect_filled(control, CONTROL.y / 2.0, fill);
    }

    response
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
