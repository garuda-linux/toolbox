mod unload;

use std::sync::Arc;

use egui::{Align, Galley, Label, Layout, Ui, vec2};
use toolbox_core::Toggle;
use toolbox_core::features::network::{
    Category, DriverFilter, DriverRow, DriverState, ModuleFacts, driver_rows, visible,
};
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::widgets::{
    ButtonKind, ListItem, RowHeights, RowList, Tone, empty_state, empty_state_action, line,
    named_chips, pill_button, push_height, rich, row_height, split_row, status_dot,
    text_block_height_with,
};
use toolbox_gui_kit::{UiExt as _, icons, tokens};
pub(super) use unload::forget;

use crate::command::NetworkCommand;
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::PageContext;
use crate::pages::rows::more::{item, more};

const CONTROLS_WIDTH: f32 = tokens::Sizes::DEFAULT.row_actions_narrow;

const STATES: [Option<DriverState>; 4] = [
    None,
    Some(DriverState::Loaded),
    Some(DriverState::Blacklisted),
    Some(DriverState::Available),
];

const KINDS: [Option<Category>; 4] = [
    None,
    Some(Category::Ethernet),
    Some(Category::Wireless),
    Some(Category::Other),
];

#[derive(Debug)]
struct Item(DriverRow);

impl ListItem for Item {
    fn key(&self) -> &str {
        self.0.name.as_str()
    }

    fn name(&self) -> &str {
        self.0.name.as_str()
    }
}

/// The drivers the filter lets through, out of how many there are.
#[derive(Debug)]
pub struct Drivers {
    total: usize,
    shown: Vec<Item>,
}

/// `None` while the snapshot holds no drivers. Keyed by the filter's
/// state and kind besides its search, the system and the queue.
pub type Rows = RowCache<
    (
        Option<DriverState>,
        Option<Category>,
        Same<toolbox_core::Snapshot>,
        u64,
    ),
    Option<Drivers>,
>;

pub fn show(ui: &mut Ui, context: &PageContext<'_>, filter: &mut DriverFilter, rows: &mut Rows) {
    unload::question(ui.ctx(), context);
    if !listed(ui, context, Option::as_ref(&drivers(context, filter, rows))) {
        return;
    }

    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    filters(ui, filter);

    // A chip picked just now applies in this frame already.
    let drivers = drivers(context, filter, rows);
    let Some(Drivers { total, shown }) = drivers.as_ref() else {
        return;
    };

    let count = i18n::text_with(
        &ctx,
        "network-drivers-count",
        &[
            ("shown", Arg::from(shown.len())),
            ("count", Arg::from(total)),
        ],
    );

    ui.label(rich(
        ui,
        &count,
        Weight::Regular,
        SMALL,
        ui.tokens().secondary,
    ));
    ui.add_space(ui.tokens().sizes.card_gap);

    if shown.is_empty() {
        let title = text("network-drivers-no-match");
        let clear = text("network-drivers-clear");
        if empty_state_action(ui, icons::SYSTEM_SEARCH, &title, "", &clear).clicked() {
            filter.search.clear();
            filter.state = None;
            filter.category = None;
        }

        return;
    }

    let name = text("network-drivers-list");
    let height = row_height(ui, text_block_height_with(ui, 1, push_height(ui)));
    RowList::new("network-drivers", &name, RowHeights::Fixed(height)).show(
        ui,
        shown.as_slice(),
        |ui, item, _state| row(ui, context, &item.0),
    );
}

/// The drivers `filter` lets through, from the cache while nothing they
/// come from changed.
fn drivers(
    context: &PageContext<'_>,
    filter: &DriverFilter,
    rows: &mut Rows,
) -> Arc<Option<Drivers>> {
    let rest = (
        filter.state,
        filter.category,
        Same::of(context.snapshot),
        context.revision,
    );

    rows.get(&[&filter.search], rest, || {
        let _build = tracing::trace_span!("build rows").entered();
        let facts = context.snapshot.part::<ModuleFacts>()?;
        let all = driver_rows(facts, context.changes);
        let shown = visible(&all, filter)
            .into_iter()
            .cloned()
            .map(Item)
            .collect();

        Some(Drivers {
            total: all.len(),
            shown,
        })
    })
}

/// Whether there are drivers, else the empty state that says why not.
fn listed(ui: &mut Ui, context: &PageContext<'_>, drivers: Option<&Drivers>) -> bool {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let (icon, title, detail) = match drivers {
        _ if !context.probed => (icons::NETWORK_WIRED, "network-drivers-reading", None),
        None => (icons::HELP_ABOUT, "network-drivers-unreadable", None),
        Some(found) if found.total == 0 => (
            icons::HELP_ABOUT,
            "network-drivers-none",
            Some("network-drivers-none-detail"),
        ),
        Some(_) => return true,
    };

    empty_state(
        ui,
        icon,
        &text(title),
        &detail.map(text).unwrap_or_default(),
    );
    false
}

fn filters(ui: &mut Ui, filter: &mut DriverFilter) {
    let states = STATES.map(|state| state.map_or("network-drivers-all", state_id));
    let kinds = KINDS.map(|kind| kind.map_or("network-drivers-all", kind_id));
    let names = ["network-drivers-state", "network-drivers-kind"]
        .map(|id| i18n::text(ui.ctx(), id).into_owned());

    // Both names take one column, so the two chip rows start at one x.
    let column = names
        .iter()
        .map(|name| caption(ui, name).size().x)
        .fold(0.0_f32, f32::max);

    pick(
        ui,
        (&names[0], column),
        (&STATES, &states),
        &mut filter.state,
    );
    ui.add_space(ui.tokens().sizes.control_gap);
    pick(
        ui,
        (&names[1], column),
        (&KINDS, &kinds),
        &mut filter.category,
    );
    ui.add_space(ui.tokens().sizes.card_gap);
}

fn caption(ui: &Ui, name: &str) -> Arc<Galley> {
    line(ui, name, Weight::Medium, SMALL, ui.tokens().secondary)
}

/// One chip row behind its visible name, `column` wide. The group keeps the
/// name too, so each row's "All" tells which row it belongs to.
fn pick<T>(
    ui: &mut Ui,
    (name, column): (&str, f32),
    (values, ids): (&[T], &[&str]),
    current: &mut T,
) where
    T: Copy + PartialEq,
{
    let ctx = ui.ctx().clone();
    let names: Vec<String> = ids
        .iter()
        .map(|id| i18n::text(&ctx, id).into_owned())
        .collect();

    let options: Vec<(T, &str)> = values
        .iter()
        .copied()
        .zip(names.iter().map(String::as_str))
        .collect();

    ui.horizontal(|ui| {
        let height = caption(ui, name).size().y;
        let layout = Layout::left_to_right(Align::Center);
        ui.allocate_ui_with_layout(vec2(column, height), layout, |ui| {
            ui.set_min_width(column);
            ui.label(rich(ui, name, Weight::Medium, SMALL, ui.tokens().secondary));
        });
        ui.add_space(ui.tokens().sizes.control_gap);

        if let Some(picked) = named_chips(ui, name, &options, *current).value {
            *current = picked;
        }
    });
}

fn row(ui: &mut Ui, context: &PageContext<'_>, driver: &DriverRow) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    split_row(
        ui,
        CONTROLS_WIDTH,
        |ui| {
            toolbox_gui_kit::widgets::title_line(ui, false, |ui| {
                let name = rich(
                    ui,
                    driver.name.as_str(),
                    Weight::SemiBold,
                    BODY,
                    tokens.text,
                );

                ui.add(Label::new(name).truncate());

                let blacklisted = driver.own || !driver.other_files.is_empty();
                let state = if driver.loaded && blacklisted {
                    "network-driver-loaded-blacklisted"
                } else {
                    state_id(driver.state)
                };

                let tone = if driver.loaded {
                    Tone::Success
                } else {
                    Tone::Neutral
                };

                status_dot(ui, &i18n::text(&ctx, state), tone);

                // A kind is no state, so it gets no dot.
                let kind = i18n::text(&ctx, kind_id(driver.category));
                ui.label(rich(ui, &kind, Weight::Regular, SMALL, tokens.secondary));
            });

            // An empty line would hold the name above its button.
            let detail = detail(&ctx, driver);
            if !detail.is_empty() {
                ui.add(
                    Label::new(rich(ui, &detail, Weight::Regular, SMALL, tokens.secondary))
                        .truncate(),
                );
            }
        },
        |ui| buttons(ui, context, driver),
    );
}

/// The queued change, else the other files that blacklist the driver.
fn detail(ctx: &egui::Context, driver: &DriverRow) -> String {
    match driver.queued {
        Some(Toggle::On) => i18n::text(ctx, "network-driver-queued-on").into_owned(),
        Some(Toggle::Off) => i18n::text(ctx, "network-driver-queued-off").into_owned(),
        None if !driver.other_files.is_empty() => i18n::text_with(
            ctx,
            "network-driver-other-files",
            &[(
                "files",
                Arg::from(
                    &driver
                        .other_files
                        .iter()
                        .map(toolbox_text::OutsideText::as_str)
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
            )],
        )
        .into_owned(),
        None => String::new(),
    }
}

/// Load or Unload, which run at once, and the blacklist, which queues, in
/// the ⋯ menu.
fn buttons(ui: &mut Ui, context: &PageContext<'_>, driver: &DriverRow) {
    load_button(ui, context, driver);

    // A driver only another file blacklists is not the app's to change.
    if !driver.own && !driver.other_files.is_empty() {
        return;
    }

    let id = match driver.queued {
        Some(_) => "network-driver-unqueue",
        None if driver.own => "network-driver-allow",
        None => "network-driver-blacklist",
    };

    more(ui, driver.name.as_str(), |ui| {
        if item(ui, id, true) {
            let wanted = driver.queued.map_or(driver.own, Toggle::is_on);
            context.commands.send(NetworkCommand::Blacklist {
                module: driver.name.clone(),
                on: Toggle::from_on(!wanted),
            });
        }
    });
}

fn load_button(ui: &mut Ui, context: &PageContext<'_>, driver: &DriverRow) {
    let ctx = ui.ctx().clone();
    let load = if driver.loaded {
        "network-driver-unload-ask"
    } else {
        "network-driver-load"
    };

    let loaded = ui
        .enabled_ui(!context.busy, |ui| {
            pill_button(ui, &i18n::text(&ctx, load), ButtonKind::Soft)
        })
        .inner;

    if !loaded.clicked() {
        return;
    }

    if driver.loaded {
        unload::ask(&ctx, &driver.name);
    } else {
        context.commands.send(NetworkCommand::LoadDriver {
            module: driver.name.clone(),
            load: true,
        });
    }
}

const fn state_id(state: DriverState) -> &'static str {
    match state {
        DriverState::Loaded => "network-driver-loaded",
        DriverState::Blacklisted => "network-driver-blacklisted",
        DriverState::Available => "network-driver-available",
    }
}

const fn kind_id(kind: Category) -> &'static str {
    match kind {
        Category::Ethernet => "network-kind-ethernet",
        Category::Wireless => "network-kind-wireless",
        Category::Other => "network-kind-other",
    }
}
