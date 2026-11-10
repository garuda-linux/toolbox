use egui::Ui;
use toolbox_core::features::maintenance::page::{
    CHAOTIC_MIRRORS, OperationRow, ToolRow, UPGRADE_SYSTEM, asks_first, operations,
};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::icons::{self, Icon};
use toolbox_gui_kit::widgets::{ButtonKind, card, card_row, loading_label};

use super::{heading, plan_button, row};
use crate::command::MaintenanceCommand;
use crate::i18n;
use crate::maintenance::Maintenance;
use crate::pages::PageContext;
use crate::pages::rows::more::{item, more};

/// `asking` holds the task whose question is open.
pub fn show(ui: &mut Ui, context: &PageContext<'_>, asking: &mut Option<String>) {
    heading(ui, icons::PACKAGE, "maintenance-operations");
    card(ui, |ui| {
        for (index, operation) in operations(context.changes).iter().enumerate() {
            card_row(ui, index == 0, |ui| {
                operation_row(ui, context, operation, asking);
            });
        }
    });

    if let Some(task) = asking.clone() {
        super::repairs::ask(ui.ctx(), context, &task, asking);
    }

    ui.add_space(ui.tokens().sizes.section_gap);
    heading(ui, icons::TOOLS, "maintenance-tools");
    let Some(programs) = context.home.programs else {
        loading_label(ui, &i18n::text(ui.ctx(), "maintenance-tools"));
        return;
    };

    card(ui, |ui| {
        for (index, tool) in programs.tools.iter().enumerate() {
            card_row(ui, index == 0, |ui| tool_row(ui, context, tool));
        }
    });
}

fn operation_row(
    ui: &mut Ui,
    context: &PageContext<'_>,
    operation: &OperationRow,
    asking: &mut Option<String>,
) {
    let ctx = ui.ctx().clone();
    let id = format!("maintenance-op-{}", operation.operation);
    let detail = i18n::text(&ctx, &format!("{id}-detail"));
    row(
        ui,
        operation_icon(operation.operation),
        &id,
        &detail,
        |ui| {
            let kind = if operation.operation == UPGRADE_SYSTEM {
                ButtonKind::Primary
            } else {
                ButtonKind::Soft
            };

            if plan_button(ui, context, "maintenance-run-now", kind) {
                if asks_first(operation.operation) {
                    *asking = Some(operation.operation.to_owned());
                } else {
                    let run = MaintenanceCommand::RunNow(operation.operation.to_owned());
                    context.commands.send(run);
                }
            }

            let queue = if operation.queued {
                "maintenance-unqueue".to_owned()
            } else {
                format!("{id}-queue")
            };

            let title = i18n::text(&ctx, &id);
            more(ui, &title, |ui| {
                if item(ui, &queue, !context.busy) {
                    context.commands.send(MaintenanceCommand::Queue {
                        operation: operation.operation.to_owned(),
                        on: !operation.queued,
                    });
                }
            });
        },
    );
}

/// A missing tool says so below its own detail, and its button installs
/// it first.
fn tool_row(ui: &mut Ui, context: &PageContext<'_>, tool: &ToolRow) {
    let ctx = ui.ctx().clone();
    let id = format!("maintenance-tool-{}", tool.tool);
    let mut detail = i18n::text(&ctx, &format!("{id}-detail")).into_owned();
    let ranking = tool.tool == CHAOTIC_MIRRORS && context.service::<Maintenance>().ranking();
    let note = if ranking {
        Some("maintenance-ranking")
    } else if tool.installed {
        None
    } else if tool.tool == CHAOTIC_MIRRORS {
        Some("maintenance-ranker-missing")
    } else {
        Some("maintenance-tool-missing")
    };

    if let Some(note) = note {
        detail.push('\n');
        detail.push_str(&i18n::text(&ctx, note));
    }

    let button = if tool.installed {
        "maintenance-open"
    } else {
        "maintenance-install-open"
    };

    row(ui, tool_icon(tool.tool), &id, &detail, |ui| {
        let clicked = ui
            .add_enabled_ui(!ranking, |ui| {
                plan_button(ui, context, button, ButtonKind::Soft)
            })
            .inner;

        if clicked {
            context
                .commands
                .send(MaintenanceCommand::OpenTool(tool.tool.to_owned()));
        }
    });
}

fn operation_icon(operation: &str) -> Icon {
    match operation {
        UPGRADE_SYSTEM => icons::SYSTEM_SOFTWARE_UPDATE,
        "remove-orphans" => icons::VIEW_LIST_DETAILS,
        "clean-package-cache" => icons::EDIT_CLEAR_ALL,
        "refresh-keyring" => icons::SECURITY_HIGH,
        _ => icons::DOCUMENT_REVERT,
    }
}

fn tool_icon(tool: &str) -> Icon {
    match tool {
        "arch-mirrors" => icons::NETWORK_WIRED,
        CHAOTIC_MIRRORS => icons::NETWORK_CONNECT,
        "btrfs-assistant" => icons::COMPUTER,
        "edit-repositories" => icons::DOCUMENT_PROPERTIES,
        _ => icons::VCS_DIFF,
    }
}
