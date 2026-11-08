use egui::{Label, Ui};
use toolbox_core::Toggle;
use toolbox_core::features::users::listing::GroupOrder;
use toolbox_core::features::users::rows::GroupRow;
use toolbox_core::features::users::view::{self, Shown};
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::widgets::{
    ButtonKind, ListItem, RowHeights, RowList, Tone, empty_state, pill_button, rich, row_height,
    split_row, status_dot, text_block_height,
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

struct Item(GroupRow);

impl ListItem for Item {
    fn key(&self) -> &str {
        self.0.name.as_str()
    }

    fn name(&self) -> &str {
        self.0.name.as_str()
    }
}

pub fn show(
    ui: &mut Ui,
    context: &PageContext<'_>,
    shown: &mut Shown,
    dialog: &mut Option<Dialog>,
) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let known = view::accounts_known(context.snapshot);
    toolbar(ui, dialog, known);
    if !accounts_ready(ui, context, "groups-reading", "groups-unavailable") {
        return;
    }

    let rows = view::groups(context.snapshot, context.changes, shown);
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

    let items: Vec<Item> = rows.into_iter().map(Item).collect();
    let name = text("groups-list");
    let heights = RowHeights::Fixed(row_height(ui, text_block_height(ui, 1)));
    RowList::new("groups-list", &name, heights).show(ui, &items, |ui, item, _state| {
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

/// Remove, or take the queued change back. A system group has none.
fn button(ui: &mut Ui, context: &PageContext<'_>, group: &GroupRow, dialog: &mut Option<Dialog>) {
    let ctx = ui.ctx().clone();
    let name = group.name.as_str();
    let (id, ask) = match group.queued {
        Some(Toggle::On) => ("groups-unqueue-new", false),
        Some(Toggle::Off) => ("groups-unqueue-removal", false),
        None if group.system => return,
        None => ("groups-remove", true),
    };

    let label = i18n::text_with(&ctx, id, &[("group", name)]);
    if !pill_button(ui, &label, ButtonKind::Soft).clicked() {
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
