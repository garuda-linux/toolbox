mod unload;

use egui::{Label, Ui};
use toolbox_core::Toggle;
use toolbox_core::features::network::{
    Category, DriverFilter, DriverRow, DriverState, ModuleFacts, driver_rows, visible,
};
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::widgets::{
    ButtonKind, ListItem, RowHeights, RowList, Tone, empty_state, named_chips, pill_button, rich,
    row_height, split_row, status_dot, text_block_height,
};
use toolbox_gui_kit::{UiExt as _, icons, tokens};
pub(super) use unload::forget;

use crate::command::NetworkCommand;
use crate::i18n;
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

struct Item<'rows>(&'rows DriverRow);

impl ListItem for Item<'_> {
    fn key(&self) -> &str {
        self.0.name.as_str()
    }

    fn name(&self) -> &str {
        self.0.name.as_str()
    }
}

pub fn show(ui: &mut Ui, context: &PageContext<'_>, filter: &mut DriverFilter) {
    unload::question(ui.ctx(), context);
    let Some(rows) = rows(ui, context) else {
        return;
    };

    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    filters(ui, filter);
    let shown = visible(&rows, filter);
    let count = i18n::text_with(
        &ctx,
        "network-drivers-count",
        &[
            ("shown", &shown.len().to_string()),
            ("count", &rows.len().to_string()),
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
        empty_state(
            ui,
            icons::SYSTEM_SEARCH,
            &text("network-drivers-no-match"),
            "",
        );
        if pill_button(ui, &text("network-drivers-clear"), ButtonKind::Soft).clicked() {
            filter.search.clear();
        }

        return;
    }

    let items: Vec<Item<'_>> = shown.into_iter().map(Item).collect();
    let name = text("network-drivers-list");
    let height = row_height(ui, text_block_height(ui, 1));
    RowList::new("network-drivers", &name, RowHeights::Fixed(height)).show(
        ui,
        &items,
        |ui, item, _state| row(ui, context, item.0),
    );
}

/// The rows, or `None` after an empty state that says why there are none.
fn rows(ui: &mut Ui, context: &PageContext<'_>) -> Option<Vec<DriverRow>> {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let facts = context.snapshot.part::<ModuleFacts>();
    let rows = facts.map(|facts| driver_rows(facts, context.changes));
    let (icon, title, detail) = match &rows {
        _ if !context.probed => (icons::NETWORK_WIRED, "network-drivers-reading", None),
        None => (icons::HELP_ABOUT, "network-drivers-unreadable", None),
        Some(found) if found.is_empty() => (
            icons::HELP_ABOUT,
            "network-drivers-none",
            Some("network-drivers-none-detail"),
        ),
        Some(_) => return rows,
    };

    empty_state(
        ui,
        icon,
        &text(title),
        &detail.map(text).unwrap_or_default(),
    );
    None
}

fn filters(ui: &mut Ui, filter: &mut DriverFilter) {
    let states = STATES.map(|state| state.map_or("network-drivers-all", state_id));
    let kinds = KINDS.map(|kind| kind.map_or("network-drivers-all", kind_id));
    pick(
        ui,
        "network-drivers-state",
        (&STATES, &states),
        &mut filter.state,
    );
    ui.add_space(ui.tokens().sizes.control_gap);
    pick(
        ui,
        "network-drivers-kind",
        (&KINDS, &kinds),
        &mut filter.category,
    );
    ui.add_space(ui.tokens().sizes.card_gap);
}

fn pick<T>(ui: &mut Ui, label: &str, (values, ids): (&[T], &[&str]), current: &mut T)
where
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

    if let Some(picked) = named_chips(ui, &i18n::text(&ctx, label), &options, *current) {
        *current = picked;
    }
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

                status_dot(
                    ui,
                    &i18n::text(&ctx, kind_id(driver.category)),
                    Tone::Neutral,
                );
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
            &[("files", &driver.other_files.join(", "))],
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
                on: !wanted,
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
        .add_enabled_ui(!context.busy, |ui| {
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
