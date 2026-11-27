//! The boot menu card: the default entry, remembering the last one, the
//! timeout and how the menu is built.

#[cfg(test)]
mod tests;

use std::sync::Arc;

use egui::{Align, Layout, Ui};
use toolbox_core::features::boot::edit::Edit;
use toolbox_core::features::boot::view::{self, EntryRow, Page};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::widgets::{Select, SpinTexts, Tone, card, spin_field};
use toolbox_plan::{GrubKey, GrubTimeout};
use toolbox_services::boot_menu::Menu;
use toolbox_system::MenuRead;
use toolbox_system::pkexec::Refusal;

use super::{heading, queued, send};
use crate::boot::Boot;
use crate::command::BootCommand;
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::PageContext;
use crate::pages::rows::{Look, read_again_banner};
use crate::pages::setting_row::{Words, row, switch_row, wide_control};

/// The timeout's limits, in seconds.
pub(super) const TIMEOUT_MAX: u32 = 300;
/// The seconds a timeout gets when it turns from a mode into seconds.
pub(super) const FIRST_SECONDS: u32 = 5;
/// The fallback of a remembered default: the first entry.
const FIRST_ENTRY: &str = "0";
/// Room for the timeout's spin field, steppers and unit included.
pub(super) const TIMEOUT_WIDTH: f32 = 120.0;
const TIMEOUT_CONTROL_WIDTH: f32 = 400.0;
const TIMEOUT_SELECT_WIDTH: f32 = 160.0;

/// The default entry's choices as entry and name, keyed by the shown default,
/// whether the menu lists the default, and the look.
pub type EntryRows = RowCache<(Same<Menu>, bool, Look), Vec<(String, String)>>;

pub fn show(ui: &mut Ui, context: &PageContext<'_>, page: &Page, entries: &mut EntryRows) {
    let ctx = ui.ctx().clone();
    heading(ui, "boot-menu");
    let menu = context.service::<Boot>().menu();
    if unread(menu) {
        unread_banner(ui, context, menu);
        ui.add_space(ui.tokens().sizes.card_gap);
    }

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

/// A default the menu does not list comes first, so it still shows.
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
            let unlisted =
                i18n::text_with(&ctx, "boot-entry-unlisted", &[("entry", Arg::from(&value))]);

            rows.insert(0, (value.clone(), unlisted.into_owned()));
        }

        rows
    });

    let description = if remembered {
        i18n::text(&ctx, "boot-default-remembered")
    } else if unread(menu) {
        i18n::text(&ctx, "boot-default-known")
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
            .enabled_ui(enabled, |ui| {
                Select::new(&title, &names)
                    .selected(if remembered { None } else { current })
                    .placeholder(if remembered { &last } else { "" })
                    .width(width)
                    .show(ui)
            })
            .inner;

        if let Some((entry, _)) = picked.value.and_then(|index| rows.get(index)) {
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

/// Seconds or a menu that waits for a key, the two ways GRUB reads its
/// number.
fn timeout(ui: &mut Ui, context: &PageContext<'_>, page: &Page) {
    let ctx = ui.ctx().clone();
    let changes = context.changes;
    let seconds = page.timeout().seconds();
    let description = if page.timeout_known() {
        i18n::text(&ctx, "boot-timeout-detail")
    } else {
        i18n::text(&ctx, "boot-timeout-unknown")
    };

    let modes = [
        i18n::text(&ctx, "boot-sdboot-timeout-seconds"),
        i18n::text(&ctx, "boot-sdboot-timeout-menu-force"),
    ];

    let names: Vec<&str> = modes.iter().map(|mode| &**mode).collect();
    let title = i18n::text(&ctx, "boot-timeout");
    let tag = queued(&ctx, view::is_queued(changes, GrubKey::Timeout));
    let words = Words {
        current: None,
        title: &title,
        description: &description,
        note: None,
        queued: tag.as_deref(),
    };

    let (reserved, width) = timeout_widths(ui, seconds.is_some());
    row(ui, false, &words, reserved, |ui| {
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            let picked = Select::new(&title, &names)
                .selected(Some(usize::from(seconds.is_none())))
                .width(width)
                .show(ui)
                .value;

            let wanted = match (picked, seconds) {
                (Some(0), None) => Some(i64::from(FIRST_SECONDS)),
                (Some(1), Some(_)) => Some(i64::from(GrubTimeout::WaitForKey)),
                (_, Some(now)) => {
                    let mut value = now.min(TIMEOUT_MAX);
                    seconds_field(ui, &ctx, &mut value).then_some(i64::from(value))
                }
                (_, None) => None,
            };

            if let Some(wanted) = wanted {
                send(context, Edit::Timeout(wanted));
            }
        });
    });
}

/// The room a timeout row reserves and its dropdown's width: as wide as
/// the other dropdowns, unless the seconds need room beside it.
pub(super) fn timeout_widths(ui: &Ui, in_seconds: bool) -> (f32, f32) {
    if in_seconds {
        (TIMEOUT_CONTROL_WIDTH, TIMEOUT_SELECT_WIDTH)
    } else {
        let wide = wide_control(ui);
        (wide, wide)
    }
}

/// Why the menu could not be read, with Read again where a second read
/// can work.
pub(super) fn unread_banner(ui: &mut Ui, context: &PageContext<'_>, menu: &Menu) {
    let futile = matches!(
        menu,
        Menu::Read(MenuRead::Refused(Refusal::NoAgent | Refusal::HelperMissing))
    );

    let said = i18n::text(ui.ctx(), &format!("boot-default-{}", menu.as_str()));
    if read_again_banner(ui, Tone::Warning, &said, !futile) {
        context.commands.send(BootCommand::Refresh);
    }
}

/// A menu that was asked for and not read. A missing one is no failure:
/// GRUB has not built it yet.
const fn unread(menu: &Menu) -> bool {
    matches!(
        menu,
        Menu::Read(
            MenuRead::Refused(_)
                | MenuRead::Unknown(_)
                | MenuRead::Unreadable(_)
                | MenuRead::Cancelled
        )
    )
}

/// Returns whether they changed.
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
        salt: "boot-timeout",
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

fn label(ctx: &egui::Context, row: &EntryRow) -> String {
    if row.parent.is_empty() {
        return row.title.clone();
    }

    i18n::text_with(
        ctx,
        "boot-entry-in",
        &[
            ("title", Arg::from(&row.title)),
            ("parent", Arg::from(&row.parent)),
        ],
    )
    .into_owned()
}
