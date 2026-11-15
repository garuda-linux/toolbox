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
use toolbox_gui_kit::widgets::{Select, SpinTexts, card, spin_field};
use toolbox_plan::GrubKey;
use toolbox_services::boot_menu::Menu;

use super::{heading, queued, send};
use crate::boot::Boot;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::rows::Look;
use crate::pages::setting_row::{Words, row, switch_row, wide_control};

/// The timeout's limits, in seconds.
pub(super) const TIMEOUT_MAX: u32 = 300;
/// The fallback of a remembered default: the first entry.
const FIRST_ENTRY: &str = "0";
/// The timeout's spin field: its two round steppers and "300 s" between.
pub(super) const TIMEOUT_WIDTH: f32 = 120.0;

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
    let menu = context.service::<Boot>().menu();
    let changes = context.changes;
    let remembered = page.remember();
    let value = page.entry_value(menu.known());
    let listed = page.entry_listed(menu.known());
    let rest = (
        context.service::<Boot>().menu_identity(),
        listed,
        Look::of(ui),
    );

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

    let width = wide_control(ui);
    row(ui, true, &words, width, |ui| {
        let names: Vec<&str> = rows.iter().map(|(_, name)| name.as_str()).collect();
        let current = rows.iter().position(|(entry, _)| *entry == value);
        let last = i18n::text(&ctx, "boot-default-last");
        let enabled = !remembered && !rows.is_empty();
        let picked = ui
            .add_enabled_ui(enabled, |ui| {
                Select::new(&title, &names)
                    .selected(if remembered { None } else { current })
                    .placeholder(if remembered { &last } else { "" })
                    .width(width)
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
    let menu = context.service::<Boot>().menu().known();
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
        let mut value = seconds;
        if seconds_field(ui, &ctx, &mut value) {
            send(context, Edit::Timeout(i64::from(value)));
        }
    });
}

/// The timeout's seconds as one spin field. Returns whether they changed.
pub(super) fn seconds_field(ui: &mut Ui, ctx: &egui::Context, value: &mut u32) -> bool {
    let texts = [
        "boot-timeout-seconds",
        "boot-timeout-unit",
        "boot-timeout-less",
        "boot-timeout-more",
    ]
    .map(|id| i18n::text(ctx, id));

    let [name, unit, less, more] = &texts;
    let spin = SpinTexts {
        name,
        unit,
        less,
        more,
    };

    spin_field(ui, value, 0..=TIMEOUT_MAX, TIMEOUT_WIDTH, spin)
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
