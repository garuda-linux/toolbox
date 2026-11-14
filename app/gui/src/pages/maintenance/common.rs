use egui::Ui;
use toolbox_core::features::maintenance::page::{OperationRow, PageTool, ToolRow, operations};
use toolbox_core::features::maintenance::{Operation, Tool};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::icons::{self, Icon};
use toolbox_gui_kit::widgets::{ActionRow, ButtonKind, Tone, card, card_row, loading_label};

use super::repairs::Asked;
use super::{heading, plan_button, row};
use crate::command::MaintenanceCommand;
use crate::i18n;
use crate::maintenance::Maintenance;
use crate::pages::PageContext;
use crate::pages::rows::more::{item, more};

/// `asking` holds the task whose question is open.
pub fn show(ui: &mut Ui, context: &PageContext<'_>, asking: &mut Option<Asked>) {
    heading(ui, icons::PACKAGE, "maintenance-operations");
    card(ui, |ui| {
        for (index, operation) in operations(context.changes).iter().enumerate() {
            card_row(ui, index == 0, |ui| {
                operation_row(ui, context, operation, asking);
            });
        }
    });

    if let Some(asked) = *asking {
        super::repairs::ask(ui.ctx(), context, asked, asking);
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
    asking: &mut Option<Asked>,
) {
    let ctx = ui.ctx().clone();
    let id = operation.operation.text_id();
    let detail = i18n::text(&ctx, &format!("{id}-detail"));
    let title = i18n::text(&ctx, &id);
    let queued = if operation.queued {
        i18n::text(&ctx, "maintenance-queued")
    } else {
        "".into()
    };

    ActionRow::new(operation_icon(operation.operation), Tone::Neutral, &title)
        .detail(&detail)
        .queued(&queued)
        .show(ui, |ui| {
            let kind = if operation.operation == Operation::UpgradeSystem {
                ButtonKind::Primary
            } else {
                ButtonKind::Soft
            };

            if plan_button(ui, context, "maintenance-run-now", kind) {
                if operation.operation.asks_first() {
                    *asking = Some(Asked::Operation(operation.operation));
                } else {
                    let run = MaintenanceCommand::RunOperation(operation.operation);
                    context.commands.send(run);
                }
            }

            let queue = if operation.queued {
                "maintenance-unqueue".to_owned()
            } else {
                format!("{id}-queue")
            };

            more(ui, &title, |ui| {
                if item(ui, &queue, !context.busy) {
                    context.commands.send(MaintenanceCommand::Queue {
                        operation: operation.operation,
                        on: !operation.queued,
                    });
                }
            });
        });
}

/// A missing tool says so below its own detail, and its button installs
/// it first.
fn tool_row(ui: &mut Ui, context: &PageContext<'_>, tool: &ToolRow) {
    let ctx = ui.ctx().clone();
    let id = tool.tool.text_id();
    let mut detail = i18n::text(&ctx, &format!("{id}-detail")).into_owned();
    let mirrors = tool.tool == PageTool::ChaoticMirrors;
    let ranking = mirrors && context.service::<Maintenance>().ranking();
    let note = if ranking {
        Some("maintenance-ranking")
    } else if tool.installed {
        None
    } else if mirrors {
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
                .send(MaintenanceCommand::OpenTool(tool.tool));
        }
    });
}

const fn operation_icon(operation: Operation) -> Icon {
    match operation {
        Operation::UpgradeSystem => icons::SYSTEM_SOFTWARE_UPDATE,
        Operation::RemoveOrphans => icons::VIEW_LIST_DETAILS,
        Operation::CleanPackageCache => icons::EDIT_CLEAR_ALL,
        Operation::RefreshKeyring => icons::SECURITY_HIGH,
        Operation::RemoveDatabaseLock => icons::DOCUMENT_REVERT,
    }
}

const fn tool_icon(tool: PageTool) -> Icon {
    match tool {
        PageTool::Program(Tool::ArchMirrors) => icons::NETWORK_WIRED,
        PageTool::ChaoticMirrors => icons::NETWORK_CONNECT,
        PageTool::Program(Tool::BtrfsAssistant) => icons::COMPUTER,
        PageTool::Program(Tool::EditRepositories) => icons::DOCUMENT_PROPERTIES,
        PageTool::Program(Tool::MergePacdiff) => icons::VCS_DIFF,
    }
}
