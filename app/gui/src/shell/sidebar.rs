//! The page list on the left. Collapsed it shows only the icons, each still
//! named for Orca. When the entries do not fit the sidebar's height, the
//! gaps between the groups shrink and the list scrolls, while the entries
//! keep their height, as a smaller target is hard to hit.

mod density;
mod edges;
mod head;
mod reading;

use egui::{Frame, Margin, Panel, Rect, Ui, UiBuilder, pos2};
use toolbox_gui_kit::theme::{SIDEBAR_WIDTH, show_scroll, show_scroll_hidden};
use toolbox_gui_kit::widgets::{
    EdgeFade, HIGHLIGHT_TOP, NAV_ICON, group_label, group_label_height, nav_divider,
    nav_divider_height, nav_row,
};
use toolbox_gui_kit::{UiExt as _, motion};

use self::density::Density;
use self::edges::{
    cut_edges, entries_end, list_area, reveal, reveal_due, scroll_by, section_hides, side_edge,
    strength,
};
use self::head::{keep_entries_right, rail_toggle, search};
use crate::command::{CommandSender, UiCommand};
use crate::i18n;
use crate::page::{Group, PageId};
use crate::page_ui::{self, PageLook as _};

const EXPANDED_SALT: &str = "sidebar";
/// How many frames the open page's entry may take to scroll into place.
const REVEAL_FRAMES: u8 = 12;

/// What [`pages`] drew that the list around it needs.
struct Listed {
    /// How far the list scrolls to show the open page.
    scroll: Option<f32>,
    /// Where the entries end above a bottom cut edge whose next heading,
    /// divider or entry hides, so the fade starts there.
    end: Option<f32>,
    /// The open page's entry, which no fade covers.
    open: Option<Rect>,
    /// The collapsed rail draws an entry cut by the bottom edge. No fade
    /// covers it there, its own keeps the cut icon in sight.
    cut: bool,
}

/// `instant`: the shortcut changed `expanded`, so the sidebar jumps.
/// `reading`: a read runs that no status bar shows.
pub fn show(
    ui: &mut Ui,
    (current, reading): (PageId, bool),
    expanded: &mut bool,
    instant: bool,
    commands: &CommandSender,
) {
    let tokens = ui.tokens();
    let padding = tokens.sizes.sidebar_padding;
    let frame = Frame::new().fill(tokens.sidebar).inner_margin(padding);

    // An entry's own 2 px margins are its frame's, so the entries reach the
    // sidebar's edges, as Breeze's item views do. The sidebar's own lines
    // stay `NAV_INSET` in.
    let flush = Frame::new().fill(tokens.sidebar).inner_margin(Margin {
        left: 0,
        right: 0,
        ..padding
    });

    let collapsed_panel = Panel::left("sidebar-collapsed")
        .resizable(false)
        .show_separator_line(false)
        .frame(frame)
        .exact_size(tokens.sizes.sidebar_collapsed);

    // egui keys the switch's slide to the expanded panel's id.
    if instant {
        let slide = ui.scope_id().with(EXPANDED_SALT).with("animation");
        motion::settle(ui.ctx(), slide, *expanded);
    }

    let expanded_panel = Panel::left(EXPANDED_SALT)
        .resizable(false)
        .show_separator_line(false)
        .frame(flush)
        .exact_size(SIDEBAR_WIDTH);

    let sizes = tokens.sizes;
    Panel::show_switched(
        ui,
        expanded,
        collapsed_panel,
        expanded_panel,
        |ui, show_labels| {
            ui.name_pane(&i18n::text(ui.ctx(), "pane-sidebar"));
            if !show_labels {
                rail_toggle(ui, commands);
            }

            // The reading note is a row of its own below the list, so the
            // list packs into what it leaves.
            let reading_row = if reading { sizes.nav_row_height } else { 0.0 };
            let available = ui.available_height() - reading_row;
            let density = Density::fitting(ui, &sizes, show_labels, available);
            search(ui, density, show_labels, commands);
            ui.add_space(density.top_gap);

            pages_and_foot(ui, density, (current, reading), commands);
        },
    );
    side_edge(ui);
}

/// The page list and the app's own pages at its foot, with the reading
/// note beside them.
fn pages_and_foot(
    ui: &mut Ui,
    density: Density,
    (current, reading): (PageId, bool),
    commands: &CommandSender,
) {
    // The foot keeps its room first and draws after the list, so the
    // keyboard meets the pages in the order they show.
    let label = group_label_height(ui, density.compact);
    let foot = Panel::bottom("sidebar-app")
        .show_separator_line(false)
        .frame(Frame::NONE)
        .exact_size(density.pages_height(ui.ctx(), label, true))
        .show(ui, |ui| ui.max_rect())
        .inner;

    if reading {
        reading::show(ui, density.row, density.expanded);
    }

    // `cut_edges` lays the fade in the sidebar's own fill, egui's would come
    // on top.
    ui.spacing_mut().scroll.fade.strength = 0.0;
    let height = density.pages_height(ui.ctx(), label, false);

    // Read every frame, also while an app page is open and no entry of the
    // list is, so a return to the same page reveals it again.
    if reveal_due(ui, current, reading) {
        ui.data_mut(|data| data.insert_temp(revealing_id(), REVEAL_FRAMES));
    }

    let area = list_area(ui);
    let content = |inside: &mut Ui| {
        keep_entries_right(inside, inside.max_rect().right());
        let fade = EdgeFade::of(inside, height, density.row);
        pages(inside, current, density, commands, Some(fade))
    };

    let list = if density.expanded {
        show_scroll(ui, area, content)
    } else {
        show_scroll_hidden(ui, area, content)
    };

    if let Some(by) = list.inner.scroll {
        scroll_by(ui, &list, by);
    }

    cut_edges(ui, &list, density.row, density.gap);

    // The list keeps its scroll strip beside the entries when it is
    // expanded, so the foot ends where the list's entries end.
    let foot = Rect::from_min_max(foot.min, pos2(list.inner_rect.right(), foot.max.y));
    ui.scope_builder(UiBuilder::new().max_rect(foot), |ui| {
        _ = pages(ui, current, density, commands, None);
    });
}

/// Every group in the scrolled `list`, its entries faded at a cut edge, or
/// the app's own pages at the foot without one.
fn pages(
    ui: &mut Ui,
    current: PageId,
    density: Density,
    commands: &CommandSender,
    list: Option<EdgeFade>,
) -> Listed {
    let foot = list.is_none();
    let rail = !foot && !density.expanded;
    let fade = list.unwrap_or_else(|| EdgeFade::of(ui, 0.0, density.row));
    ui.spacing_mut().item_spacing.y = density.gap;
    let start = ui.cursor().top();
    let listed: Vec<PageId> = PageId::ALL
        .into_iter()
        .filter(|page| (page.group() == Group::App) == foot && page_ui::listed(ui.ctx(), *page))
        .collect();

    // Where each heading or entry stands at the top, the room not yet known.
    let mut tops = Vec::new();
    let mut room = None;
    let mut open = None;
    let mut above = None;
    let mut end = None;
    let mut cut = false;
    let mut group = None;

    for (index, page) in listed.iter().copied().enumerate() {
        if group != Some(page.group()) {
            group = Some(page.group());
            let hidden = section_hides(ui, &fade, ui.cursor().top(), page.group(), density);
            let shown = section_start(ui, &fade, page.group(), density, hidden);

            // A heading or divider at the top stands where an entry's
            // middle would, so it keeps the room the entries keep. In a
            // packed list that would cut the entry above, so it is no stop.
            let floor = above
                .zip(room)
                .map_or(f32::MIN, |(bottom, gap)| bottom + gap - 1.5);

            let top = shown.map(|middle| middle - density.row / 2.0);
            tops.extend(top.filter(|top| *top >= floor));
        }

        // The open page's entry keeps its strength, it names the page in
        // sight.
        let ends = (page != current)
            .then(|| entries_end(ui, &fade, page, listed.get(index + 1).copied(), density))
            .flatten();

        end = end.or(ends);

        let rows = highlight_rows(ui.cursor().top(), density.row);
        let name = i18n::text(ui.ctx(), page.text_id());

        let opacity = rail_strength(ui, &fade, rows, rail, (page == current, ends, &mut cut));
        let row = faded(ui, opacity, |ui| {
            nav_row(ui, page.icon(), &name, page == current, density.look())
        });

        if row.clicked() {
            commands.send(UiCommand::Navigate(page));
        }

        room.get_or_insert_with(|| row.rect.top() - start);
        tops.push(row.rect.top());
        above = Some(row.rect.bottom());
        if page == current {
            open = Some(row.rect);
        }
    }

    let open = open.filter(|_| !foot);
    let scroll = scroll_to_stop(ui, &fade, (tops, room), open);

    Listed {
        scroll,
        end,
        open,
        cut,
    }
}

/// How far the list scrolls to show the `open` entry, once the stops are
/// known. `tops` hold each stop from the list's top, `room` is the room
/// above the first one at rest.
fn scroll_to_stop(
    ui: &mut Ui,
    fade: &EdgeFade,
    (tops, room): (Vec<f32>, Option<f32>),
    open: Option<Rect>,
) -> Option<f32> {
    let stops: Vec<f32> = tops
        .into_iter()
        .map(|top| top - room.unwrap_or(0.0))
        .collect();

    if fade.is_cut() {
        room_past_the_end(ui, &stops);
    }

    scroll_to_open(ui, &stops, open?)
}

/// The span of an entry's highlight in a row `top` down and `row` tall. The
/// fades judge the entry by its highlight, the part that shows.
fn highlight_rows(top: f32, row: f32) -> egui::Rangef {
    egui::Rangef::new(top + HIGHLIGHT_TOP, top + row)
}

/// [`strength`] of an entry over `rows`, which sets `cut` when the `rail`
/// draws it cut by the bottom edge. A cut icon with at least a quarter
/// showing draws faded, so the rail shows that more follows. Less reads as a
/// speck, so it hides.
fn rail_strength(
    ui: &Ui,
    fade: &EdgeFade,
    rows: egui::Rangef,
    rail: bool,
    (open, ends, cut): (bool, Option<f32>, &mut bool),
) -> f32 {
    let bottom = ui.clip_rect().bottom();
    let icon_top = rows.min + (rows.span() - NAV_ICON) / 2.0;
    if rail && bottom - icon_top < NAV_ICON / 4.0 {
        return 0.0;
    }

    *cut |= rail && rows.max > bottom + 0.5;
    strength(ui, fade, rows, (open, ends, rail))
}

/// A cut list scrolls on by up to one stop past its last entry, so the
/// last entries too can show with a whole one at the top.
fn room_past_the_end(ui: &mut Ui, stops: &[f32]) {
    let step = stops
        .iter()
        .zip(stops.iter().skip(1))
        .map(|(one, next)| next - one);

    ui.allocate_space(egui::vec2(0.0, step.fold(0.0, f32::max)));
}

fn scroll_to_open(ui: &Ui, stops: &[f32], entry: Rect) -> Option<f32> {
    let id = revealing_id();

    // The sidebar's layout settles over the first frames, so the entry
    // keeps scrolling into place until it holds. Then the user may scroll
    // away.
    let left = ui.data(|data| data.get_temp::<u8>(id)).unwrap_or(0);
    let by = (left > 0).then(|| reveal(ui, stops, entry)).flatten();
    ui.data_mut(|data| match by {
        Some(_) => _ = data.insert_temp(id, left - 1),
        None => _ = data.remove_temp::<u8>(id),
    });
    by
}

/// How many frames the open page's entry may still scroll into place, set
/// in [`show`] once it became the open page.
fn revealing_id() -> egui::Id {
    egui::Id::unique("toolbox_gui::sidebar::revealing")
}

/// A heading above a named group, or a little room while collapsed, at
/// its opacity in the `fade`, or not at all when `hidden`. Returns the
/// middle of the heading or divider it drew.
fn section_start(
    ui: &mut Ui,
    fade: &EdgeFade,
    group: Group,
    density: Density,
    hidden: bool,
) -> Option<f32> {
    let opacity = |at: &Ui, height: f32| {
        let top = at.cursor().top();
        if hidden {
            0.0
        } else {
            fade.opacity(top, top + height)
        }
    };

    match group.heading_id() {
        Some(heading) if density.expanded => {
            ui.add_space(density.label_room);
            let height = group_label_height(ui, density.compact);
            let middle = faded(ui, opacity(ui, height), |ui| {
                group_label(ui, &i18n::text(ui.ctx(), heading), density.compact);

                // Only a packed list scrolls, and packed the heading's row
                // is its text alone.
                let text = group_label_height(ui, true);
                Some(ui.min_rect().bottom() - text / 2.0)
            });

            ui.add_space(density.label_below);
            middle
        }
        Some(_) => {
            let height = nav_divider_height(density.divider);
            faded(ui, opacity(ui, height), |ui| {
                nav_divider(ui, density.divider);
                Some(ui.min_rect().center().y)
            })
        }
        None => fade.show(ui, density.group_gap, |ui| {
            ui.add_space(density.group_gap);
            None
        }),
    }
}

fn faded<R>(ui: &mut Ui, opacity: f32, entry: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope(|ui| {
        ui.multiply_opacity(opacity);
        entry(ui)
    })
    .inner
}
