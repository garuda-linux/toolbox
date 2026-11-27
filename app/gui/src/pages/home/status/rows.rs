use egui::Ui;
use toolbox_core::features::maintenance::page::{PageTool, installed, operations};
use toolbox_core::features::maintenance::{Operation, RunNow, Tool, upgrade_queued};
use toolbox_gui_kit::widgets::{ActionRow, ButtonKind, pill_button, text_button, text_link};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_plan::UnitName;
use toolbox_services::home::Overview;
use toolbox_system::HealthLevel;
use toolbox_text::size;

use super::super::HomePage;
use super::{run_button, run_row};
use crate::command::{HomeCommand, MaintenanceCommand, UiCommand};
use crate::home::Home;
use crate::i18n;
use crate::i18n::Arg;
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
    let count = overview.updates;
    let title = i18n::text_with(&ctx, "home-updates", &[("count", Arg::from(count))]);
    let detail = i18n::text(&ctx, "home-updates-detail");

    let queued = upgrade_queued(context.changes);
    let queue = if queued {
        "home-unqueue-update"
    } else {
        "home-queue-update"
    };

    let queue = i18n::text(&ctx, queue);
    ActionRow::new(icons::SYSTEM_SOFTWARE_UPDATE, &title)
        .detail(&detail)
        .show_with_body(
            ui,
            |ui| {
                ui.horizontal(|ui| {
                    if text_button(ui, &queue, &queue).clicked() {
                        context.commands.send(HomeCommand::QueueUpgrade(!queued));
                    }

                    show_packages(page, ui);
                });
            },
            |ui| {
                run_button(
                    ui,
                    context,
                    "home-update-now",
                    RunNow::UpgradeSystem,
                    ButtonKind::Primary,
                );
            },
        );
}

pub(super) fn aur(
    page: &mut HomePage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    overview: &Overview<'_>,
) {
    let ctx = ui.ctx().clone();
    let count = overview.aur_updates;
    let title = i18n::text_with(&ctx, "home-aur-updates", &[("count", Arg::from(count))]);
    let detail = i18n::text(&ctx, "home-aur-detail");
    ActionRow::new(icons::PACKAGE, &title)
        .detail(&detail)
        .show_with_body(
            ui,
            |ui| show_packages(page, ui),
            |ui| {
                run_button(
                    ui,
                    context,
                    "home-aur-update",
                    RunNow::AurUpgrade,
                    ButtonKind::Soft,
                );
            },
        );
}

fn show_packages(page: &mut HomePage, ui: &mut Ui) {
    let text = i18n::text(ui.ctx(), "home-show-packages");
    if text_link(ui, &text, &text).clicked() {
        page.updates_open = true;
    }
}

pub(super) fn stale(ui: &mut Ui, context: &PageContext<'_>) {
    run_row(
        ui,
        context,
        icons::CHRONOMETER,
        ["home-stale", "home-stale-detail", "home-update-now"],
        RunNow::UpgradeSystem,
    );
}

/// Review opens the dialog that compares and queues them. meld, through
/// the script Maintenance runs, stays in the menu while it is installed.
pub(super) fn merges(
    page: &mut HomePage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    overview: &Overview<'_>,
) {
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, "home-merges");
    let meld = installed(
        context.service::<Home>().tools(),
        PageTool::Program(Tool::MergePacdiff),
    );

    let files: Vec<String> = overview
        .merges
        .iter()
        .map(|file| file.display().to_string())
        .collect();

    ActionRow::new(icons::VCS_DIFF, &title)
        .detail(&files.join("\n"))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let review = i18n::text(&ctx, "home-merges-review");
                if pill_button(ui, &review, ButtonKind::Soft).clicked() {
                    super::super::merges::open(&mut page.merges, context);
                }

                if !meld {
                    return;
                }

                more(ui, &title, |ui| {
                    if item(ui, "home-merges-meld", !context.busy) {
                        context
                            .commands
                            .send(MaintenanceCommand::OpenTool(PageTool::Program(
                                Tool::MergePacdiff,
                            )));
                    }
                });
            });
        });
}

pub(super) fn health(
    ui: &mut Ui,
    context: &PageContext<'_>,
    overview: &Overview<'_>,
    level: HealthLevel,
) {
    let ctx = ui.ctx().clone();
    let (title, icon) = if level >= HealthLevel::High {
        ("home-health-errors", icons::DIALOG_ERROR)
    } else {
        ("home-health-warnings", icons::DIALOG_WARNING)
    };

    let detail = overview.health_messages.join("\n");
    ActionRow::new(icon, &i18n::text(&ctx, title))
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
    ActionRow::new(icons::SYSTEM_REBOOT, &title)
        .detail(&detail)
        .show(ui, |ui| {
            let restart = i18n::text(&ctx, "home-restart-button");
            let pressed = ui
                .enabled_ui(!context.busy, |ui| {
                    pill_button(ui, &restart, ButtonKind::Soft)
                })
                .inner;

            if pressed.clicked() {
                page.asking_to_restart = true;
            }
        });
}

pub(super) fn disk(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, "home-disk");
    let detail = i18n::text(&ctx, "home-disk-detail");
    ActionRow::new(icons::DIALOG_WARNING, &title)
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
    let count = overview.machine.failed_units.len();
    let title = i18n::text_with(&ctx, "home-services", &[("count", Arg::from(count))]);
    let units: Vec<&str> = overview
        .machine
        .failed_units
        .iter()
        .map(UnitName::as_str)
        .collect();

    ActionRow::new(icons::DIALOG_ERROR, &title)
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
    let orphans = overview.machine.orphans;
    let cache = size(overview.machine.package_cache);
    let detail = i18n::text_with(
        &ctx,
        if overview.machine.orphans == 0 {
            "home-cleanup-detail-cache"
        } else {
            "home-cleanup-detail"
        },
        &[
            ("orphans", Arg::from(orphans)),
            ("cache", Arg::from(&cache)),
        ],
    );

    let cleanup = [Operation::RemoveOrphans, Operation::CleanPackageCache];
    let queued = operations(context.changes)
        .iter()
        .any(|row| row.queued && cleanup.contains(&row.operation));

    ActionRow::new(icons::EDIT_CLEAR_ALL, &title)
        .detail(&detail)
        .show(ui, |ui| {
            let id = if queued {
                "home-cleanup-unqueue"
            } else {
                "home-cleanup-queue"
            };

            if pill_button(ui, &i18n::text(&ctx, id), ButtonKind::Soft).clicked() {
                for operation in cleanup {
                    context.commands.send(MaintenanceCommand::Queue {
                        operation,
                        on: toolbox_core::Toggle::from_on(!queued),
                    });
                }
            }
        });
}
