use egui::{Label, Ui};
use toolbox_core::features::users::listing::GroupOrder;
use toolbox_core::features::users::rows::GroupRow;
use toolbox_core::features::users::view::{self, Shown};
use toolbox_core::{Snapshot, Toggle};
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::widgets::{
    ButtonKind, ListItem, RowHeights, RowList, Tone, empty_state, named_pill_button, pill_button,
    rich, row_height, split_row, status_dot, text_block_height,
};
use toolbox_gui_kit::{UiExt as _, icons};

use super::dialogs::{AddGroup, Dialog, RemoveGroup};
use super::{accounts_ready, texts};
use crate::command::UserEdit;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::rows::sort_select;

const ORDERS: [(GroupOrder, &str); 3] = [
    (GroupOrder::Name, "groups-order-name"),
    (GroupOrder::Id, "groups-order-id"),
    (GroupOrder::Members, "groups-order-members"),
];

const CONTROLS_WIDTH: f32 = toolbox_gui_kit::tokens::Sizes::DEFAULT.row_actions_narrow;

#[derive(Debug)]
pub struct Item(GroupRow);

impl ListItem for Item {
    fn key(&self) -> &str {
        self.0.name.as_str()
    }

    fn name(&self) -> &str {
        self.0.name.as_str()
    }
}

/// Keyed by the order besides the search, the system and the queue.
pub type Rows = RowCache<(GroupOrder, Same<Snapshot>, u64), Vec<Item>>;

pub fn show(
    ui: &mut Ui,
    context: &PageContext<'_>,
    (shown, cache): (&mut Shown, &mut Rows),
    dialog: &mut Option<Dialog>,
) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let known = view::accounts_known(context.snapshot);
    toolbar(ui, dialog, known);
    let ids = ("groups-reading", "groups-unavailable");
    if !accounts_ready(ui, context, ids, group_height(ui)) {
        return;
    }

    let rest = (
        shown.group_order,
        Same::of(context.snapshot),
        context.revision,
    );

    let rows = cache.get(&[&shown.group_search], rest, || {
        let _build = tracing::trace_span!("build rows").entered();
        view::groups(context.snapshot, context.changes, shown)
            .into_iter()
            .map(Item)
            .collect()
    });

    if rows.is_empty() {
        let title = i18n::text_with(&ctx, "groups-no-match", &[("search", &shown.group_search)]);
        empty_state(
            ui,
            icons::SYSTEM_SEARCH,
            &title,
            &text("groups-search-detail"),
        );
        if pill_button(ui, &text("users-clear-search"), ButtonKind::Soft).clicked() {
            shown.group_search.clear();
        }

        return;
    }

    let name = text("groups-list");
    let heights = RowHeights::Fixed(group_height(ui));
    RowList::new("groups-list", &name, heights).show(ui, rows.as_slice(), |ui, item, _state| {
        row(ui, context, &item.0, dialog);
    });
}

fn toolbar(ui: &mut Ui, dialog: &mut Option<Dialog>, known: bool) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    ui.horizontal(|ui| {
        let add = ui
            .add_enabled_ui(known, |ui| {
                pill_button(ui, &text("groups-add"), ButtonKind::Primary)
            })
            .inner;

        if add.clicked() {
            *dialog = Some(Dialog::AddGroup(AddGroup::default()));
        }
    });

    ui.add_space(ui.tokens().sizes.card_gap);
}

fn group_height(ui: &Ui) -> f32 {
    row_height(ui, text_block_height(ui, 1))
}

fn row(ui: &mut Ui, context: &PageContext<'_>, group: &GroupRow, dialog: &mut Option<Dialog>) {
    let ctx = ui.ctx().clone();
    let tokens = ui.tokens();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    split_row(
        ui,
        CONTROLS_WIDTH,
        |ui| {
            toolbox_gui_kit::widgets::title_line(ui, false, |ui| {
                let title = rich(ui, group.name.as_str(), Weight::SemiBold, BODY, tokens.text);
                ui.add(Label::new(title).truncate());
                match group.queued {
                    Some(Toggle::On) => status_dot(ui, &text("groups-tag-new"), Tone::Accent),
                    Some(Toggle::Off) => status_dot(ui, &text("groups-tag-removed"), Tone::Accent),
                    None if group.system => {
                        status_dot(ui, &text("groups-tag-system"), Tone::Neutral)
                    }
                    None => ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover()),
                };
            });

            let detail = detail(&ctx, group);
            ui.add(
                Label::new(rich(ui, &detail, Weight::Regular, SMALL, tokens.secondary)).truncate(),
            );
        },
        |ui| button(ui, context, group, dialog),
    );
}

fn detail(ctx: &egui::Context, group: &GroupRow) -> String {
    let Some(gid) = group.gid else {
        return i18n::text(ctx, "groups-detail-new").into_owned();
    };

    let gid = gid.to_string();
    if group.members.is_empty() {
        return i18n::text_with(ctx, "groups-detail-empty", &[("gid", &gid)]).into_owned();
    }

    let members = texts::list(ctx, group.members.iter().map(toolbox_plan::Name::as_str));
    i18n::text_with(
        ctx,
        "groups-detail",
        &[("gid", &gid), ("members", &members)],
    )
    .into_owned()
}

/// Remove, or take the queued change back. A system group and an
/// account's primary group have none, as the removal would be refused.
fn button(ui: &mut Ui, context: &PageContext<'_>, group: &GroupRow, dialog: &mut Option<Dialog>) {
    let ctx = ui.ctx().clone();
    let (id, ask) = match group.queued {
        Some(Toggle::On) => ("groups-unqueue-new", false),
        Some(Toggle::Off) => ("groups-unqueue-removal", false),
        None if group.system || group.primary => return,
        None => ("groups-remove", true),
    };

    // Every row shows the same words, so the column stays straight and
    // each button names its group for a screen reader.
    let label = i18n::text(&ctx, id);
    let name = i18n::text_with(
        &ctx,
        &format!("{id}-named"),
        &[("group", group.name.as_str())],
    );

    if !named_pill_button(ui, &label, ButtonKind::Soft, &name).clicked() {
        return;
    }

    if ask {
        *dialog = Some(Dialog::RemoveGroup(RemoveGroup(group.name.clone())));
    } else if group.queued == Some(Toggle::On) {
        context
            .commands
            .send(UserEdit::RemoveGroup(group.name.clone()));
    } else {
        context
            .commands
            .send(UserEdit::AddGroup(group.name.clone()));
    }
}

pub fn sort(ui: &mut Ui, shown: &mut Shown) {
    sort_select(ui, "groups-sort", &ORDERS, &mut shown.group_order);
}
