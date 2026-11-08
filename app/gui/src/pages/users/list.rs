use egui::Ui;
use toolbox_core::features::users::listing::UserOrder;
use toolbox_core::features::users::rows::UserRow;
use toolbox_core::features::users::view::{self, Shown};
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::widgets::{
    ButtonKind, ListItem, RowHeights, RowList, chip, empty_state, pill_button, rich,
};
use toolbox_gui_kit::{UiExt as _, icons};

use super::dialogs::Dialog;
use super::{accounts_ready, row};
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::rows::sort_select;

const ORDERS: [(UserOrder, &str); 5] = [
    (UserOrder::Name, "users-order-name"),
    (UserOrder::Id, "users-order-id"),
    (UserOrder::FullName, "users-order-full-name"),
    (UserOrder::Home, "users-order-home"),
    (UserOrder::Shell, "users-order-shell"),
];

/// A row as the list shows it, named for screen readers by its user and
/// full name.
pub struct Item<'rows> {
    row: UserRow<'rows>,
    name: String,
}

impl ListItem for Item<'_> {
    fn key(&self) -> &str {
        self.row.user.name.as_str()
    }

    fn name(&self) -> &str {
        &self.name
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
    toolbar(ui, context, shown, dialog, known);
    if !accounts_ready(ui, context, "users-reading", "users-unavailable") {
        return;
    }

    let rows = view::users(context.snapshot, context.changes, shown);
    let total = view::users_total(context.snapshot).to_string();
    let count = i18n::text_with(
        &ctx,
        "users-count",
        &[("shown", &rows.len().to_string()), ("count", &total)],
    );

    ui.label(rich(
        ui,
        &count,
        Weight::Regular,
        SMALL,
        ui.tokens().secondary,
    ));
    ui.add_space(ui.tokens().sizes.card_gap);
    if rows.is_empty() {
        nobody(ui, shown);
        return;
    }

    let items: Vec<Item<'_>> = rows
        .into_iter()
        .map(|row| Item {
            name: format!("{}, {}", row.user.name, row.shown_full_name()),
            row,
        })
        .collect();

    let name = text("users-list");
    let heights = RowHeights::Fixed(row::height(ui));
    RowList::new("users-list", &name, heights).show(ui, &items, |ui, item, state| {
        row::show(ui, context, (&item.row, state), dialog);
    });

    ui.add_space(ui.tokens().sizes.card_gap);
    ui.label(rich(
        ui,
        &text("users-keys"),
        Weight::Regular,
        SMALL,
        ui.tokens().secondary,
    ));
}

fn toolbar(
    ui: &mut Ui,
    context: &PageContext<'_>,
    shown: &mut Shown,
    dialog: &mut Option<Dialog>,
    known: bool,
) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    ui.horizontal(|ui| {
        let add = ui
            .add_enabled_ui(known && !context.busy, |ui| {
                pill_button(ui, &text("users-add"), ButtonKind::Primary)
            })
            .inner;

        if add.clicked() {
            *dialog = Some(Dialog::AddUser(Box::default()));
        }

        ui.add_space(ui.tokens().sizes.card_gap);
        let label = text("users-regular-only");
        if chip(ui, &label, None, shown.regular_only)
            .on_hover_text(text("users-regular-only-detail"))
            .clicked()
        {
            shown.regular_only = !shown.regular_only;
        }
    });

    ui.add_space(ui.tokens().sizes.card_gap);
}

fn nobody(ui: &mut Ui, shown: &mut Shown) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    if shown.user_search.is_empty() {
        let detail = text("users-none-regular-detail");
        empty_state(ui, icons::USER_OTHERS, &text("users-none-regular"), &detail);
        if pill_button(ui, &text("users-show-all"), ButtonKind::Soft).clicked() {
            shown.regular_only = false;
        }

        return;
    }

    let title = i18n::text_with(&ctx, "users-no-match", &[("search", &shown.user_search)]);
    empty_state(
        ui,
        icons::SYSTEM_SEARCH,
        &title,
        &text("users-search-detail"),
    );
    if pill_button(ui, &text("users-clear-search"), ButtonKind::Soft).clicked() {
        shown.user_search.clear();
    }
}

pub fn sort(ui: &mut Ui, shown: &mut Shown) {
    sort_select(ui, "users-sort", &ORDERS, &mut shown.user_order);
}
