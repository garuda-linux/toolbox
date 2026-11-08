//! The edges of the sidebar's scrolled list: where it is cut and where it
//! scrolls the open page to.

use egui::scroll_area::ScrollAreaOutput;
use egui::{Color32, Direction, Id, Mesh, Rect, ScrollArea, Shape, Ui, pos2};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{EdgeFade, group_label_height, nav_divider_height};

use super::Listed;
use super::density::Density;
use crate::page::{Group, PageId};

/// How far the list scrolls to show the open page's `entry`, `None` when
/// it is in place. The list then starts at the first of its `stops` that
/// still shows the entry whole. A stop is a group or an entry with the
/// room the list has above it at rest, so no entry is cut at the top.
pub(super) fn reveal(ui: &Ui, stops: &[f32], entry: Rect) -> Option<f32> {
    let view = ui.clip_rect();
    let lowest = entry.bottom() - view.height();
    let top = stops
        .iter()
        .copied()
        .find(|stop| *stop >= lowest - 0.5)
        .map_or_else(|| entry.top(), |stop| stop.min(entry.top()));

    let by = top - view.top();
    (by.abs() > 0.5).then_some(by)
}

/// The sidebar's list, at the offset [`scroll_by`] left for it.
pub(super) fn list_area(ui: &Ui) -> ScrollArea {
    let area = ScrollArea::vertical();
    match ui.data_mut(|data| data.remove_temp::<f32>(offset_id(ui))) {
        Some(offset) => area.vertical_scroll_offset(offset),
        None => area,
    }
}

/// Scrolls the `list` by `by` on the next frame, exactly. egui's scroll
/// target would land off by the item spacing it keeps around the target.
pub(super) fn scroll_by<R>(ui: &Ui, list: &ScrollAreaOutput<R>, by: f32) {
    let most = (list.content_size.y - list.inner_rect.height()).max(0.0);
    let offset = (list.state.clamped_offset().y + by).clamp(0.0, most);
    ui.data_mut(|data| data.insert_temp(offset_id(ui), offset));
    ui.ctx().request_repaint();
}

fn offset_id(ui: &Ui) -> Id {
    ui.scope_id().with("sidebar list offset")
}

/// Whether the heading or divider that starts `group`'s section at
/// `start` hides: a heading cut by an edge, or either one whose first
/// entry lies past the bottom edge, as it would part nothing.
pub(super) fn section_hides(
    ui: &Ui,
    fade: &EdgeFade,
    start: f32,
    group: Group,
    density: Density,
) -> bool {
    let (room, height) = match group.heading_id() {
        Some(_) if density.expanded => {
            (density.label_room, group_label_height(ui, density.compact))
        }
        Some(_) => (0.0, nav_divider_height(density.divider)),
        None => return false,
    };

    let top = start + room;
    let entry = top + height + density.gap;
    fade.cuts(top, top + height) || past_the_end(ui, entry, density)
}

/// Whether an entry from `top` lies past the bottom edge. Expanded that is
/// its middle, as the fade covers all but a sliver of it. The collapsed
/// rail shows any part of it, cut and faded, so its icons run on to the
/// edge and show that more follows.
fn past_the_end(ui: &Ui, top: f32, density: Density) -> bool {
    let shown = if density.expanded {
        density.row / 2.0
    } else {
        0.0
    };

    top + shown >= ui.clip_rect().bottom() - 0.5
}

/// Where the entries end when what follows `page`'s entry, drawn next at
/// the cursor, hides at the bottom edge: a heading, divider or entry past
/// the edge. The entry then stands apart from the edge, so it fades as if
/// the edge ran there.
pub(super) fn entries_end(
    ui: &Ui,
    fade: &EdgeFade,
    page: PageId,
    next: Option<PageId>,
    density: Density,
) -> Option<f32> {
    let view = ui.clip_rect();
    let after = ui.cursor().top() + density.row + density.gap;
    let next = next?;
    let lower = (view.center().y..view.bottom()).contains(&after);
    let hides = if next.group() == page.group() {
        past_the_end(ui, after, density)
    } else {
        section_hides(ui, fade, after, next.group(), density)
    };

    (lower && hides).then_some(after)
}

/// A soft fade into the sidebar's fill, `reach` deep, and a hairline where
/// the scrolled list is cut. The entries' own fade hardly shows between
/// icons. Neither fade reaches the open page's entry or, at the bottom, an
/// entry the rail draws cut, which would hide it whole.
pub(super) fn cut_edges(ui: &Ui, list: &ScrollAreaOutput<Listed>, reach: f32) {
    let Listed {
        scroll: _,
        end,
        open,
        cut,
    } = list.inner;

    let offset = list.state.clamped_offset().y;
    let hidden_below = list.content_size.y - offset - list.inner_rect.height();
    let rect = list.inner_rect;
    let tokens = ui.tokens();
    let inset = tokens.sizes.inset;
    let hairline = egui::Stroke::new(1.0, tokens.sidebar_edge);
    let span = (rect.left() + inset)..=(rect.right() - inset);

    if offset > 0.5 {
        let clear = open.filter(|open| open.top() >= rect.top()).map_or_else(
            || rect.top() + reach,
            |open| open.top().min(rect.top() + reach),
        );

        let band = Rect::from_min_max(rect.left_top(), pos2(rect.right(), clear));
        let colours = [tokens.sidebar, Color32::TRANSPARENT];
        ui.painter()
            .add(Shape::gradient_rect(band, Direction::TopDown, colours));
        ui.painter().hline(span.clone(), rect.top() + 0.5, hairline);
    }

    if hidden_below > 0.5 {
        let bottom = rect.bottom();
        let whole = end.map_or(bottom, |end| end.clamp(rect.top(), bottom));
        let clear = open
            .filter(|open| open.bottom() <= bottom)
            .map_or(whole - reach, |open| open.bottom().max(whole - reach));

        let whole = whole.max(clear);
        let mut mesh = Mesh::default();
        for (y, colour) in [
            (clear, Color32::TRANSPARENT),
            (whole, tokens.sidebar),
            (rect.bottom(), tokens.sidebar),
        ] {
            mesh.colored_vertex(pos2(rect.left(), y), colour);
            mesh.colored_vertex(pos2(rect.right(), y), colour);
        }

        for first in [0, 2] {
            mesh.add_triangle(first, first + 1, first + 2);
            mesh.add_triangle(first + 2, first + 1, first + 3);
        }

        if !cut {
            ui.painter().add(mesh);
        }

        ui.painter().hline(span, rect.bottom() - 0.5, hairline);
    }
}

/// A hairline down the sidebar's side, on its last pixel column, so the
/// page and its header band start right beside it. In light the sidebar
/// lies too near the page for its fill alone to part them.
pub(super) fn side_edge(ui: &Ui) {
    let page = ui.available_rect_before_wrap();
    let pixel = 1.0 / ui.pixels_per_point();
    let x = ui
        .painter()
        .round_to_pixel_center(page.left() - pixel / 2.0);

    let hairline = egui::Stroke::new(pixel, ui.tokens().sidebar_edge);
    ui.painter().vline(x, page.y_range(), hairline);
}

/// Whether `current` became the open page since the sidebar last drew,
/// so its entry scrolls into sight once and the user may scroll away.
pub(super) fn newly_current(ui: &Ui, current: PageId) -> bool {
    let id = ui.scope_id().with("current page");
    let before = ui.data(|data| data.get_temp::<PageId>(id));
    ui.data_mut(|data| data.insert_temp(id, current));
    before != Some(current)
}
