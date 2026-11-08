//! The page list on the left. Collapsed it shows only the icons, each still
//! named for Orca. When the entries do not fit the sidebar's height, the
//! gaps between the groups shrink and the list scrolls, while the entries
//! keep their height, as a smaller target is hard to hit.

mod density;
mod edges;

use egui::{Frame, Panel, Ui, UiBuilder};
use toolbox_gui_kit::fonts::Weight;
use toolbox_gui_kit::theme::SIDEBAR_WIDTH;
use toolbox_gui_kit::widgets::{
    EdgeFade, group_label, group_label_height, nav_divider, nav_divider_height, nav_row, rich,
    sidebar_button,
};
use toolbox_gui_kit::{UiExt as _, icons, motion, tokens};

use self::density::Density;
use self::edges::{
    cut_edges, entries_end, list_area, newly_current, reveal, scroll_by, section_hides, side_edge,
};
use super::PANE_TITLE;
use crate::command::{CommandSender, UiCommand};
use crate::i18n;
use crate::page::{Group, PageId};
use crate::page_ui::{self, PageLook as _};

/// Width of the collapsed sidebar: one icon row and the panel margins.
pub const COLLAPSED_WIDTH: f32 = 60.0;
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
    open: Option<egui::Rect>,
    /// The collapsed rail draws an entry cut by the bottom edge. No fade
    /// covers it there, its own keeps the half icon in sight.
    cut: bool,
}

/// `instant`: the shortcut changed `expanded`, so the sidebar jumps.
pub fn show(
    ui: &mut Ui,
    current: PageId,
    expanded: &mut bool,
    instant: bool,
    commands: &CommandSender,
) {
    let tokens = ui.tokens();
    let frame = Frame::new()
        .fill(tokens.sidebar)
        .inner_margin(tokens.sizes.sidebar_padding);

    let collapsed_panel = Panel::left("sidebar-collapsed")
        .resizable(false)
        .show_separator_line(false)
        .frame(frame)
        .exact_size(COLLAPSED_WIDTH);

    // egui keys the switch's slide to the expanded panel's id.
    if instant {
        let slide = ui.scope_id().with(EXPANDED_SALT).with("animation");
        motion::settle(ui.ctx(), slide, *expanded);
    }

    let expanded_panel = Panel::left(EXPANDED_SALT)
        .resizable(false)
        .show_separator_line(false)
        .frame(frame)
        .exact_size(SIDEBAR_WIDTH);

    let sizes = tokens.sizes;
    Panel::show_switched(
        ui,
        expanded,
        collapsed_panel,
        expanded_panel,
        |ui, show_labels| {
            ui.name_pane(&i18n::text(ui.ctx(), "pane-sidebar"));
            title(ui, show_labels, commands);
            let density = Density::fitting(ui, &sizes, show_labels, ui.available_height());
            search(ui, density, commands);
            ui.add_space(density.top_gap);

            // The foot keeps its room first and draws after the list, so
            // the keyboard meets the pages in the order they show.
            let label = group_label_height(ui, density.compact);
            let foot = Panel::bottom("sidebar-app")
                .show_separator_line(false)
                .frame(Frame::NONE)
                .exact_size(density.pages_height(ui.ctx(), label, true))
                .show(ui, |ui| ui.max_rect())
                .inner;

            // `cut_edges` lays the fade in the sidebar's own fill, egui's
            // would come on top.
            ui.spacing_mut().scroll.fade.strength = 0.0;
            let height = density.pages_height(ui.ctx(), label, false);
            let list = list_area(ui).show(ui, |ui| {
                let fade = EdgeFade::of(ui, height, density.row);
                pages(ui, current, density, commands, Some(fade))
            });

            if let Some(by) = list.inner.scroll {
                scroll_by(ui, &list, by);
            }

            cut_edges(ui, &list, density.row);
            ui.scope_builder(UiBuilder::new().max_rect(foot), |ui| {
                _ = pages(ui, current, density, commands, None);
            });
        },
    );
    side_edge(ui);
}

/// The app's name and the collapse toggle, the same at either density.
fn title(ui: &mut Ui, expanded: bool, commands: &CommandSender) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    let (icon, text_id) = if expanded {
        (icons::SIDEBAR_COLLAPSE_LEFT, "sidebar-collapse")
    } else {
        (icons::SIDEBAR_EXPAND_LEFT, "sidebar-expand")
    };

    ui.horizontal(|ui| {
        if expanded {
            ui.add_space(tokens.sizes.inset);
            ui.label(rich(
                ui,
                &i18n::text(&ctx, "app-name"),
                Weight::Bold,
                PANE_TITLE,
                tokens.text,
            ));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                toggle(ui, icon, &i18n::text(&ctx, text_id), commands);
            });
        } else {
            toggle(ui, icon, &i18n::text(&ctx, text_id), commands);
        }
    });
}

fn search(ui: &mut Ui, density: Density, commands: &CommandSender) {
    ui.add_space(density.top_gap);
    let search = i18n::text(ui.ctx(), "sidebar-search");
    if nav_row(ui, page_ui::SEARCH_ICON, &search, false, density.look()).clicked() {
        commands.send(UiCommand::OpenPalette);
    }
}

fn toggle(ui: &mut Ui, icon: icons::Icon, text: &str, commands: &CommandSender) {
    if sidebar_button(ui, icon, text).clicked() {
        commands.send(UiCommand::ToggleSidebar);
    }
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
        let rows = egui::Rangef::new(ui.cursor().top(), ui.cursor().top() + density.row);
        let name = i18n::text(ui.ctx(), page.text_id());
        let opacity = strength(ui, &fade, rows, page == current, ends);
        cut |= !foot && !density.expanded && rows.shrink(0.5).contains(ui.clip_rect().bottom());
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

    let room = room.unwrap_or(0.0);
    let stops: Vec<f32> = tops.into_iter().map(|top| top - room).collect();
    if fade.is_cut() {
        room_past_the_end(ui, &stops);
    }

    let open = open.filter(|_| !foot);
    let scroll = open.and_then(|entry| scroll_to_open(ui, current, &stops, entry));

    Listed {
        scroll,
        end,
        open,
        cut,
    }
}

/// How opaque an entry over `rows` draws in the `fade`. The `open` page's
/// entry keeps full strength unless an edge cuts it. One whose entries
/// `ends` above something hidden fades as if the edge ran there.
fn strength(ui: &Ui, fade: &EdgeFade, rows: egui::Rangef, open: bool, ends: Option<f32>) -> f32 {
    if open && !fade.cuts(rows.min, rows.max) {
        return 1.0;
    }

    let past = ends.map_or(0.0, |ends| ui.clip_rect().bottom() - ends);
    fade.opacity(rows.min + past, rows.max + past)
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

fn scroll_to_open(ui: &Ui, current: PageId, stops: &[f32], entry: egui::Rect) -> Option<f32> {
    let id = ui.scope_id().with("revealing");
    if newly_current(ui, current) {
        ui.data_mut(|data| data.insert_temp(id, REVEAL_FRAMES));
    }

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
            faded(ui, opacity(ui, height), |ui| {
                group_label(ui, &i18n::text(ui.ctx(), heading), density.compact);

                // Only a packed list scrolls, and packed the heading's row
                // is its text alone.
                let text = group_label_height(ui, true);
                Some(ui.min_rect().bottom() - text / 2.0)
            })
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
