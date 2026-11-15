//! The Repairs tab: repairs that run in the Console, where they can ask
//! their questions. A repair whose program is missing stays listed but
//! greyed out, so the banner's install shows what it unlocks. A repair
//! that reinstalls packages or throws away what the user had asks first.

use egui::{Id, Ui};
use toolbox_core::features::maintenance::Operation;
use toolbox_core::features::maintenance::page::{Repair, RepairRow, missing};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::icons::{self, Icon};
use toolbox_gui_kit::widgets::{
    ButtonKind, DialogWidth, Question, Tone, banner, card, card_row, loading_label, pill_button,
    question, rich,
};
use toolbox_plan::{PackageName, RemoteAction};

use super::{heading, row};
use crate::command::MaintenanceCommand;
use crate::i18n;
use crate::pages::PageContext;

/// The task whose question is open. Common tasks and repairs share the
/// dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Asked {
    Operation(Operation),
    Repair(Repair),
}

impl Asked {
    fn text_id(self) -> String {
        match self {
            Self::Operation(operation) => operation.text_id(),
            Self::Repair(repair) => repair.text_id(),
        }
    }

    const fn command(self) -> MaintenanceCommand {
        match self {
            Self::Operation(operation) => MaintenanceCommand::RunOperation(operation),
            Self::Repair(repair) => MaintenanceCommand::RunRepair(repair),
        }
    }
}

pub fn show(ui: &mut Ui, context: &PageContext<'_>, asking: &mut Option<Asked>) {
    let ctx = ui.ctx().clone();
    heading(ui, icons::GAMES_SOLVE, "maintenance-repairs");
    let Some(programs) = context.home.programs else {
        loading_label(ui, &i18n::text(&ctx, "maintenance-repairs"));
        return;
    };

    missing_banner(ui, context, &programs.repairs);

    card(ui, |ui| {
        for (index, repair) in programs.repairs.iter().enumerate() {
            card_row(ui, index == 0, |ui| {
                if repair_row(ui, context, repair) {
                    let asked = Asked::Repair(repair.repair);
                    if repair.repair.asks_first() {
                        *asking = Some(asked);
                    } else {
                        context.commands.send(asked.command());
                    }
                }
            });
        }
    });

    if let Some(asked) = *asking {
        ask(&ctx, context, asked, asking);
    }
}

/// One banner for every package the greyed repairs need, so one install
/// unlocks them all.
fn missing_banner(ui: &mut Ui, context: &PageContext<'_>, rows: &[RepairRow]) {
    let wanted = missing(rows);
    if wanted.is_empty() {
        return;
    }

    let ctx = ui.ctx().clone();
    let names: Vec<&str> = wanted.into_iter().map(PackageName::as_str).collect();
    let packages = names.join(", ");
    let count = names.len().to_string();
    let args = [("packages", packages.as_str()), ("count", count.as_str())];
    let text = i18n::text_with(&ctx, "maintenance-repairs-missing", &args);
    let install = i18n::text_with(&ctx, "maintenance-install-missing", &args);
    let action = (!context.busy).then_some(&*install);
    if banner(ui, Tone::Neutral, &text, action.as_slice()) == Some(0) {
        context.commands.send(MaintenanceCommand::InstallMissing);
    }

    ui.add_space(ui.tokens().sizes.card_gap);
}

/// Returns whether its Run was clicked.
fn repair_row(ui: &mut Ui, context: &PageContext<'_>, repair: &RepairRow) -> bool {
    let id = repair.repair.text_id();
    let detail = i18n::text(ui.ctx(), &format!("{id}-detail"));
    let mut clicked = false;
    row(ui, repair_icon(repair.repair), &id, &detail, |ui| {
        // A greyed Run says what brings it.
        if !repair.available {
            let needs = i18n::text_with(
                ui.ctx(),
                "maintenance-repair-needs",
                &[("package", repair.needs.as_str())],
            );

            let secondary = ui.tokens().secondary;
            ui.label(rich(ui, &needs, Weight::Regular, SMALL, secondary));
        }

        let text = i18n::text(ui.ctx(), "maintenance-run");
        let enabled = repair.available && !context.busy;
        clicked = ui
            .add_enabled_ui(enabled, |ui| pill_button(ui, &text, ButtonKind::Soft))
            .inner
            .clicked();
    });

    clicked
}

/// The question before the task `asked`, which says what is lost. Its
/// name is the Danger button, so the answer repeats what runs.
pub(super) fn ask(
    ctx: &egui::Context,
    context: &PageContext<'_>,
    asked: Asked,
    asking: &mut Option<Asked>,
) {
    let id = asked.text_id();

    let name = i18n::text(ctx, &id);
    let title = i18n::text_with(ctx, "maintenance-repair-question", &[("repair", &name)]);
    let close = i18n::text(ctx, "dialog-close");
    let body = i18n::text(ctx, &format!("{id}-question"));
    let cancel = i18n::text(ctx, "maintenance-repair-cancel");
    let texts = Question {
        title: &title,
        close: &close,
        body: &body,
        cancel: &cancel,
        confirm: &name,
        kind: ButtonKind::Danger,
        width: DialogWidth::Small,
    };

    let asked_now = question(ctx, Id::unique("toolbox_gui::maintenance::repair"), texts);
    if asked_now.confirmed {
        context.commands.send(asked.command());
    }

    if asked_now.done {
        *asking = None;
    }
}

const fn repair_icon(repair: Repair) -> Icon {
    match repair {
        Repair::HealthCheck => icons::VIEW_TASK,
        Repair::HealthFix => icons::GAMES_HINT,
        Repair::Remote(RemoteAction::Fix) => icons::CONFIGURE,
        Repair::Remote(RemoteAction::Keyring) => icons::SECURITY_HIGH,
        Repair::Remote(RemoteAction::FullFix) => icons::RUN_BUILD,
        Repair::Remote(RemoteAction::ResetAudio) => icons::AUDIO_VOLUME_HIGH,
        Repair::Remote(RemoteAction::ResetSnapper) => icons::DOCUMENT_REVERT,
        Repair::Remote(RemoteAction::Reinstall) => icons::RUN_INSTALL,
    }
}
