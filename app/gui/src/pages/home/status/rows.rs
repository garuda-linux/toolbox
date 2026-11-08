use egui::Ui;
use toolbox_cli::output::size;
use toolbox_core::features::maintenance::page::operations;
use toolbox_core::features::maintenance::{Operation, RunNow, upgrade_queued};
use toolbox_gui_kit::icons;
use toolbox_gui_kit::widgets::{ActionRow, ButtonKind, Tone, pill_button};
use toolbox_plan::UnitName;
use toolbox_system::HealthLevel;

use super::super::HomePage;
use super::super::view::Overview;
use super::{run_button, run_row};
use crate::command::{HomeCommand, MaintenanceCommand, UiCommand};
use crate::i18n;
use crate::page::PageId;
use crate::pages::PageContext;
use crate::pages::rows::more::{item, more};

pub(super) fn updates(
    page: &mut HomePage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    overview: &Overview<'_>,
) {
    let ctx = ui.ctx().clone();
    let count = overview.updates.to_string();
    let title = i18n::text_with(&ctx, "home-updates", &[("count", &count)]);
    let detail = i18n::text(&ctx, "home-updates-detail");

    let queued = upgrade_queued(context.changes);
    ActionRow::new(icons::SYSTEM_SOFTWARE_UPDATE, Tone::Neutral, &title)
        .detail(&detail)
        .show(ui, |ui| {
            run_button(
                ui,
                context,
                "home-update-now",
                RunNow::UpgradeSystem,
                ButtonKind::Primary,
            );
            let queue = if queued {
                "home-unqueue-update"
            } else {
                "home-queue-update"
            };

            more(ui, &title, |ui| {
                if item(ui, queue, true) {
                    context.commands.send(HomeCommand::QueueUpgrade(!queued));
                }

                show_packages(page, ui);
            });
        });
}

pub(super) fn aur(
    page: &mut HomePage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    overview: &Overview<'_>,
) {
    let ctx = ui.ctx().clone();
    let count = overview.aur_updates.to_string();
    let title = i18n::text_with(&ctx, "home-aur-updates", &[("count", &count)]);
    let detail = i18n::text(&ctx, "home-aur-detail");
    ActionRow::new(icons::PACKAGE, Tone::Neutral, &title)
        .detail(&detail)
        .show(ui, |ui| {
            run_button(
                ui,
                context,
                "home-aur-update",
                RunNow::AurUpgrade,
                ButtonKind::Soft,
            );
            more(ui, &title, |ui| show_packages(page, ui));
        });
}

fn show_packages(page: &mut HomePage, ui: &mut Ui) {
    if item(ui, "home-show-packages", true) {
        page.updates_open = true;
    }
}

pub(super) fn stale(ui: &mut Ui, context: &PageContext<'_>) {
    run_row(
        ui,
        context,
        (icons::CHRONOMETER, Tone::Warning),
        ["home-stale", "home-stale-detail", "home-update-now"],
        RunNow::UpgradeSystem,
    );
}

/// The files only, their merge tool opens from Maintenance.
pub(super) fn merges(ui: &mut Ui, overview: &Overview<'_>) {
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, "home-merges");
    let files: Vec<String> = overview
        .merges
        .iter()
        .map(|file| file.display().to_string())
        .collect();

    ActionRow::new(icons::VCS_DIFF, Tone::Warning, &title)
        .detail(&files.join("\n"))
        .show(ui, |_| {});
}

pub(super) fn health(
    ui: &mut Ui,
    context: &PageContext<'_>,
    overview: &Overview<'_>,
    level: HealthLevel,
) {
    let ctx = ui.ctx().clone();
    let (title, icon, tone) = if level >= HealthLevel::High {
        ("home-health-errors", icons::DIALOG_ERROR, Tone::Error)
    } else {
        ("home-health-warnings", icons::DIALOG_WARNING, Tone::Warning)
    };

    let detail = overview.health_messages.join("\n");
    ActionRow::new(icon, tone, &i18n::text(&ctx, title))
        .detail(&detail)
        .show(ui, |ui| {
            let (id, run) = if overview.health_fixable {
                ("home-health-fix", RunNow::HealthFix)
            } else {
                ("home-health-check", RunNow::HealthCheck)
            };

            run_button(ui, context, id, run, ButtonKind::Soft);
        });
}

/// A restart in the middle of pacman can leave the system unbootable, so
/// it waits for the running plan.
pub(super) fn restart(page: &mut HomePage, ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, "home-restart");
    let detail = i18n::text(&ctx, "home-restart-detail");
    ActionRow::new(icons::SYSTEM_REBOOT, Tone::Warning, &title)
        .detail(&detail)
        .show(ui, |ui| {
            let restart = i18n::text(&ctx, "home-restart-button");
            let pressed = ui
                .add_enabled_ui(!context.busy, |ui| {
                    pill_button(ui, &restart, ButtonKind::Soft)
                })
                .inner;

            if pressed.clicked() {
                page.asking_to_restart = true;
            }
        });
}

/// Maintenance holds what frees space, with its review step.
pub(super) fn disk(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, "home-disk");
    let detail = i18n::text(&ctx, "home-disk-detail");
    ActionRow::new(icons::DIALOG_WARNING, Tone::Warning, &title)
        .detail(&detail)
        .show(ui, |ui| {
            let open = i18n::text(&ctx, "home-disk-button");
            if pill_button(ui, &open, ButtonKind::Soft).clicked() {
                context
                    .commands
                    .send(UiCommand::Navigate(PageId::Maintenance));
            }
        });
}

/// The units' names are systemd's words, shown as they are.
pub(super) fn services(ui: &mut Ui, context: &PageContext<'_>, overview: &Overview<'_>) {
    let ctx = ui.ctx().clone();
    let count = overview.machine.failed_units.len().to_string();
    let title = i18n::text_with(&ctx, "home-services", &[("count", &count)]);
    let units: Vec<&str> = overview
        .machine
        .failed_units
        .iter()
        .map(UnitName::as_str)
        .collect();

    ActionRow::new(icons::DIALOG_ERROR, Tone::Error, &title)
        .detail(&units.join("\n"))
        .show(ui, |ui| {
            let open = i18n::text(&ctx, "home-services-button");
            if pill_button(ui, &open, ButtonKind::Soft).clicked() {
                context.commands.send(UiCommand::Navigate(PageId::Services));
            }
        });
}

/// Queues removing the orphans and cleaning the cache together, or takes
/// both back, so they wait for the review like on Maintenance.
pub(super) fn cleanup(ui: &mut Ui, context: &PageContext<'_>, overview: &Overview<'_>) {
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, "home-cleanup");
    let orphans = overview.machine.orphans.to_string();
    let cache = size(overview.machine.package_cache);
    let detail = i18n::text_with(
        &ctx,
        if overview.machine.orphans == 0 {
            "home-cleanup-detail-cache"
        } else {
            "home-cleanup-detail"
        },
        &[("orphans", &orphans), ("cache", &cache)],
    );

    let keys = [
        Operation::RemoveOrphans.key(),
        Operation::CleanPackageCache.key(),
    ];

    let queued = operations(context.changes)
        .iter()
        .any(|row| row.queued && keys.contains(&row.operation));

    ActionRow::new(icons::EDIT_CLEAR_ALL, Tone::Neutral, &title)
        .detail(&detail)
        .show(ui, |ui| {
            let id = if queued {
                "home-cleanup-unqueue"
            } else {
                "home-cleanup-queue"
            };

            if pill_button(ui, &i18n::text(&ctx, id), ButtonKind::Soft).clicked() {
                for operation in keys {
                    context.commands.send(MaintenanceCommand::Queue {
                        operation: operation.to_owned(),
                        on: !queued,
                    });
                }
            }
        });
}
