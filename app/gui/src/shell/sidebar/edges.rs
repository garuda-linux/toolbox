//! The edges of the sidebar's scrolled list: where it is cut and where it
//! scrolls the open page to.

use egui::scroll_area::ScrollAreaOutput;
use egui::{Color32, Direction, Id, Mesh, Rect, ScrollArea, Shape, Ui, pos2};
use toolbox_gui_kit::widgets::{EdgeFade, group_label_height, nav_divider_height};
use toolbox_gui_kit::{UiExt as _, tokens};

use super::Listed;
use super::density::Density;
use crate::page::{Group, PageId};

/// The contrast a cut entry keeps against the sidebar once the cut's fill
/// lies fully over it. Both themes fade to this, so neither reads as the
/// faded one.
const FADED_CONTRAST: f32 = 2.0;
/// The contrast the cut starts from at its clear edge, before the entry's own
/// drawn contrast caps it. From 4.5 the last entry above the cut still read
/// too strong.
const FADE_FROM: f32 = 3.5;
/// The steps the cut's fill takes between its clear edge and its whole one.
const COVER_STEPS: usize = 32;

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
        .find(|stop| *stop >= lowest - 0.01)
        .map_or_else(|| entry.top(), |stop| stop.min(entry.top()));

    let by = top - view.top();
    (by.abs() > 0.5).then_some(by)
}

/// The sidebar's list, at the offset [`scroll_by`] left for it. The
/// collapsed rail shows no bar (see `show_scroll_hidden`), as its width is
/// the icons' alone: the wheel and the keys scroll it, and the fade and
/// hairline mark the cut.
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
    let (room, height, below) = match group.heading_id() {
        Some(_) if density.expanded => (
            density.label_room,
            group_label_height(ui, density.compact),
            density.label_below,
        ),
        Some(_) => (0.0, nav_divider_height(density.divider), 0.0),
        None => return false,
    };

    let top = start + room;
    let entry = top + height + below + density.gap;
    fade.cuts(top, top + height) || past_the_end(ui, entry, density)
}

/// Whether an entry from `top` lies past the bottom edge, by its middle.
/// Expanded the fade covers all but a sliver of it. The collapsed rail
/// shows it cut and faded while at least half its icon shows, so its
/// icons run on to the edge and show that more follows.
fn past_the_end(ui: &Ui, top: f32, density: Density) -> bool {
    top + density.row / 2.0 >= ui.clip_rect().bottom() - 0.5
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

/// The fill at each `share` of the cut's gradient, running from 0 at the
/// clear edge to 1 at the whole one. The entries under the cut draw at full
/// strength, so the fill alone sets their contrast. Each stop is solved for
/// lerp([`FADE_FROM`], [`FADED_CONTRAST`], [`eased`] `share`), never above
/// the ink's own contrast, so both themes fade alike.
fn cover_steps(ui: &Ui, fill: Color32, shares: &[f32]) -> Vec<Color32> {
    let theme = ui.tokens();
    let sidebar = theme.sidebar;
    let shown = sidebar.blend(theme.sidebar_text);
    let start = tokens::contrast(shown, sidebar);

    shares
        .iter()
        .map(|&share| {
            let contrast_at = |strength: f32| {
                tokens::contrast(shown.blend(fill.gamma_multiply(strength)), sidebar)
            };

            // The contrast falls as the fill strengthens, so a bisection finds the
            // strength that leaves `wanted`.
            let wanted = egui::lerp(FADE_FROM..=FADED_CONTRAST, eased(share)).min(start);

            // The clear edge is clear, so the fill starts without a line.
            let strength = if share <= 0.0 || contrast_at(0.0) <= wanted {
                0.0
            } else {
                let (mut weak, mut strong) = (0.0_f32, 1.0_f32);
                for _ in 0..24_u32 {
                    let middle = f32::midpoint(weak, strong);
                    if contrast_at(middle) > wanted {
                        weak = middle;
                    } else {
                        strong = middle;
                    }
                }

                strong
            };

            fill.gamma_multiply(strength)
        })
        .collect()
}

/// How far `step` lies from the clear edge to the whole one, 0 to 1.
fn share_of(step: usize) -> f32 {
    let of = |count: usize| f32::from(u16::try_from(count).unwrap_or(u16::MAX));
    of(step) / of(COVER_STEPS)
}

/// `share` bent by an ease-out, as a straight fade left the last entry above
/// the cut too strong.
fn eased(share: f32) -> f32 {
    1.0 - (1.0 - share).powi(3)
}

/// How opaque an entry over `rows` draws in the `fade`. The `open` page's
/// entry keeps full strength unless an edge cuts it. One whose entries
/// `ends` above something hidden fades as if the edge ran there.
pub(super) fn strength(
    ui: &Ui,
    fade: &EdgeFade,
    rows: egui::Rangef,
    (open, ends, rail): (bool, Option<f32>, bool),
) -> f32 {
    if open && !fade.cuts(rows.min, rows.max) {
        return 1.0;
    }

    let past = ends.map_or(0.0, |ends| ui.clip_rect().bottom() - ends);
    let (top, bottom) = (rows.min + past, rows.max + past);

    // The expanded list's bottom cut is the cover's to fade, the rail has none.
    if rail {
        fade.opacity(top, bottom)
    } else {
        fade.opacity_top_only(top, bottom)
    }
}

/// A soft fade into the sidebar's fill, `reach` deep, and a hairline where
/// the scrolled list is cut. The entries' own fade hardly shows between
/// icons. Neither fade reaches the open page's entry or, at the bottom, an
/// entry the rail draws cut, which would hide it whole.
pub(super) fn cut_edges(ui: &Ui, list: &ScrollAreaOutput<Listed>, reach: f32, gap: f32) {
    let Listed {
        scroll: _,
        end,
        open,
        cut,
    } = list.inner;

    let offset = list.state.clamped_offset().y;

    // The list adds its item spacing after the last entry too, which is no
    // entry hidden below the edge.
    let hidden_below = list.content_size.y - gap - offset - list.inner_rect.height();
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
        let colours = [tokens.sidebar_fade, Color32::TRANSPARENT];
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
        if !cut {
            ui.painter().add(cover_mesh(ui, rect, (clear, whole), open));
        }

        ui.painter().hline(span, rect.bottom() - 0.5, hairline);
    }
}

/// The cut's fill over `rect` from `clear` down to `whole`, then solid to the
/// bottom of `rect`. The open entry's edges are stops of their own, so the
/// fill is clear over the open entry and it never fades.
fn cover_mesh(ui: &Ui, rect: Rect, (clear, whole): (f32, f32), open: Option<Rect>) -> Mesh {
    let mut ys: Vec<f32> = (0..=COVER_STEPS)
        .map(|step| egui::lerp(clear..=whole, share_of(step)))
        .collect();

    if let Some(open) = open {
        ys.extend(
            [open.top(), open.bottom()]
                .into_iter()
                .filter(|y| (clear..=whole).contains(y)),
        );
    }

    ys.sort_by(f32::total_cmp);

    let width = (whole - clear).max(f32::EPSILON);
    let shares: Vec<f32> = ys
        .iter()
        .map(|&y| ((y - clear) / width).clamp(0.0, 1.0))
        .collect();

    let covers = cover_steps(ui, ui.tokens().sidebar_fade, &shares);
    let mut stops: Vec<(f32, Color32)> = ys
        .iter()
        .zip(covers)
        .map(|(&y, cover)| {
            let under_open = open.is_some_and(|open| open.top() <= y && y <= open.bottom());
            (
                y,
                if under_open {
                    Color32::TRANSPARENT
                } else {
                    cover
                },
            )
        })
        .collect();

    let whole_cover = stops.last().map_or(Color32::TRANSPARENT, |stop| stop.1);
    stops.push((rect.bottom(), whole_cover));

    let mut mesh = Mesh::default();
    for &(y, colour) in &stops {
        mesh.colored_vertex(pos2(rect.left(), y), colour);
        mesh.colored_vertex(pos2(rect.right(), y), colour);
    }

    let mut first = 0_u32;
    for _ in 1..stops.len() {
        mesh.add_triangle(first, first + 1, first + 2);
        mesh.add_triangle(first + 2, first + 1, first + 3);
        first += 2;
    }

    mesh
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
