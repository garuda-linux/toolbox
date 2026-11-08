//! The boot menu card: the default entry, remembering the last one, the
//! timeout and how the menu is built.

#[cfg(test)]
mod tests;

use std::sync::Arc;

use egui::Ui;
use toolbox_core::features::boot::edit::Edit;
use toolbox_core::features::boot::view::{self, EntryRow, Page};
use toolbox_gui_kit::icons;
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::widgets::{ButtonKind, Select, card, number_field_with_unit, pill_button};
use toolbox_plan::GrubKey;
use toolbox_system::Menu;

use super::{WIDE_CONTROL, heading, queued, send};
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::rows::Look;
use crate::pages::setting_row::{Words, row, switch_row};

/// The timeout's limits, in seconds.
const TIMEOUT_MAX: u32 = 300;
/// The fallback of a remembered default: the first entry.
const FIRST_ENTRY: &str = "0";
/// Room for the timeout's two buttons and its number.
const TIMEOUT_WIDTH: f32 = 176.0;
/// "300 s" with the field's padding, as wide as the network tools' numbers.
const TIMEOUT_FIELD_WIDTH: f32 = 72.0;

/// The default entry's choices as entry and name. The shown default keys
/// it as its text, beside the menu, whether the menu lists the default,
/// and the look.
pub type EntryRows = RowCache<(Same<Menu>, bool, Look), Vec<(String, String)>>;

pub fn show(ui: &mut Ui, context: &PageContext<'_>, page: &Page, entries: &mut EntryRows) {
    let ctx = ui.ctx().clone();
    heading(ui, icons::FORMAT_LIST_UNORDERED, "boot-menu");
    card(ui, |ui| {
        default_entry(ui, context, page, entries);
        remember(ui, context, page);
        timeout(ui, context, page);

        let changes = context.changes;
        let submenus = page.submenus();
        let queued_submenus = queued(&ctx, view::is_queued(changes, GrubKey::Submenus));
        if flipped(ui, "boot-submenus", queued_submenus.as_deref(), submenus) {
            send(context, Edit::Submenus(!submenus));
        }

        let prober = page.os_prober();
        let queued_prober = queued(&ctx, view::is_queued(changes, GrubKey::OsProber));
        if flipped(ui, "boot-os-prober", queued_prober.as_deref(), prober) {
            send(context, Edit::OsProber(!prober));
        }
    });
}

/// The entries to pick from, and the default as GRUB has it. A default
/// the menu does not list comes first, so it still shows.
fn default_entry(ui: &mut Ui, context: &PageContext<'_>, page: &Page, entries: &mut EntryRows) {
    let ctx = ui.ctx().clone();
    let menu = context.boot.menu();
    let changes = context.changes;
    let remembered = page.remember();
    let value = page.entry_value(menu.known());
    let listed = page.entry_listed(menu.known());
    let rest = (context.boot.menu_identity(), listed, Look::of(ui));
    let rows: Arc<Vec<(String, String)>> = entries.get(&[&value], rest, || {
        let mut rows: Vec<(String, String)> = view::entries(menu.known())
            .into_iter()
            .map(|row| (row.entry.clone(), label(&ctx, &row)))
            .collect();

        if !value.is_empty() && !listed {
            let unlisted = i18n::text_with(&ctx, "boot-entry-unlisted", &[("entry", &value)]);
            rows.insert(0, (value.clone(), unlisted.into_owned()));
        }

        rows
    });

    let description = if remembered {
        i18n::text(&ctx, "boot-default-remembered")
    } else {
        i18n::text(&ctx, &format!("boot-default-{}", menu.as_str()))
    };

    let title = i18n::text(&ctx, "boot-default");
    let tag = queued(&ctx, view::is_queued(changes, GrubKey::Default));
    let words = Words {
        current: None,
        title: &title,
        description: &description,
        note: None,
        queued: tag.as_deref(),
    };

    row(ui, true, &words, WIDE_CONTROL, |ui| {
        let names: Vec<&str> = rows.iter().map(|(_, name)| name.as_str()).collect();
        let current = rows.iter().position(|(entry, _)| *entry == value);
        let last = i18n::text(&ctx, "boot-default-last");
        let enabled = !remembered && !rows.is_empty();
        let picked = ui
            .add_enabled_ui(enabled, |ui| {
                Select::new(&title, &names)
                    .selected(if remembered { None } else { current })
                    .placeholder(if remembered { &last } else { "" })
                    .width(WIDE_CONTROL)
                    .show(ui)
            })
            .inner;

        if let Some((entry, _)) = picked.and_then(|index| rows.get(index)) {
            send(context, Edit::Default(entry.clone()));
        }
    });
}

/// Switching it off keeps the shown default, else the first entry.
fn remember(ui: &mut Ui, context: &PageContext<'_>, page: &Page) {
    let ctx = ui.ctx().clone();
    let changes = context.changes;
    let menu = context.boot.menu().known();
    let on = page.remember();
    let tag = queued(&ctx, view::is_queued(changes, GrubKey::SaveDefault));
    if !flipped(ui, "boot-remember", tag.as_deref(), on) {
        return;
    }

    let value = page.entry_value(menu);
    let fallback = if value.is_empty() {
        view::entries(menu)
            .first()
            .map_or_else(|| FIRST_ENTRY.to_owned(), |row| row.entry.clone())
    } else {
        value
    };

    send(context, Edit::Remember { on: !on, fallback });
}

/// Seconds, with a button for one less and one more beside the number.
fn timeout(ui: &mut Ui, context: &PageContext<'_>, page: &Page) {
    let ctx = ui.ctx().clone();
    let changes = context.changes;

    // GRUB reads a negative timeout as waiting for a key, which the field
    // cannot show, so it starts from 0.
    let seconds = u32::try_from(page.timeout()).unwrap_or(0).min(TIMEOUT_MAX);
    let description = if page.timeout_known() {
        i18n::text(&ctx, "boot-timeout-detail")
    } else {
        i18n::text(&ctx, "boot-timeout-unknown")
    };

    let title = i18n::text(&ctx, "boot-timeout");
    let tag = queued(&ctx, view::is_queued(changes, GrubKey::Timeout));
    let words = Words {
        current: None,
        title: &title,
        description: &description,
        note: None,
        queued: tag.as_deref(),
    };

    row(ui, false, &words, TIMEOUT_WIDTH, |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            let less = i18n::text(&ctx, "boot-timeout-less");
            let more = i18n::text(&ctx, "boot-timeout-more");
            let mut value = seconds;
            if step(ui, "−", &less) {
                value = value.saturating_sub(1);
            }

            let name = i18n::text(&ctx, "boot-timeout-seconds");
            let unit = i18n::text(&ctx, "boot-timeout-unit");
            number_field_with_unit(
                ui,
                &mut value,
                0..=TIMEOUT_MAX,
                TIMEOUT_FIELD_WIDTH,
                (&name, &unit),
            );
            if step(ui, "+", &more) {
                value += 1;
            }

            let value = value.min(TIMEOUT_MAX);
            if value != seconds {
                send(context, Edit::Timeout(i64::from(value)));
            }
        });
    });
}

/// A row with a switch, titled `id` and described by `<id>-detail`.
/// Returns whether it was flipped.
pub(super) fn flipped(ui: &mut Ui, id: &str, queued: Option<&str>, on: bool) -> bool {
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, id);
    let description = i18n::text(&ctx, &format!("{id}-detail"));
    let words = Words {
        current: None,
        title: &title,
        description: &description,
        note: None,
        queued,
    };

    switch_row(ui, false, &words, on, true)
}

/// An entry's name, with the submenu that holds it.
fn label(ctx: &egui::Context, row: &EntryRow) -> String {
    if row.parent.is_empty() {
        return row.title.clone();
    }

    i18n::text_with(
        ctx,
        "boot-entry-in",
        &[("title", &row.title), ("parent", &row.parent)],
    )
    .into_owned()
}

/// A small button showing `symbol`, which a screen reader calls `name`.
fn step(ui: &mut Ui, symbol: &str, name: &str) -> bool {
    let response = pill_button(ui, symbol, ButtonKind::Soft);
    let label = name.to_owned();
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_label(label));

    response.clicked()
}
