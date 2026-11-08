//! The Repairs tab: repairs that run in the Console, where they can ask
//! their questions. A repair whose program is missing stays listed but
//! greyed out, so the banner's install shows what it unlocks. A repair
//! that reinstalls packages or throws away what the user had asks first.

use egui::{Id, Ui};
use toolbox_core::features::maintenance::page::{RepairRow, asks_first};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::icons::{self, Icon};
use toolbox_gui_kit::widgets::{
    ButtonKind, DialogWidth, Question, Tone, banner, card, card_row, loading_label, pill_button,
    question, rich,
};

use super::{heading, row};
use crate::command::MaintenanceCommand;
use crate::i18n;
use crate::pages::PageContext;

pub fn show(ui: &mut Ui, context: &PageContext<'_>, asking: &mut Option<String>) {
    let ctx = ui.ctx().clone();
    heading(ui, icons::GAMES_SOLVE, "maintenance-repairs");
    let Some(programs) = context.home.programs else {
        loading_label(ui, &i18n::text(&ctx, "maintenance-repairs"));
        return;
    };

    if !programs.garuda_update {
        let text = i18n::text(&ctx, "maintenance-repairs-missing");
        let install = i18n::text(&ctx, "maintenance-install-garuda-update");
        let action = (!context.busy).then_some(&*install);
        if banner(ui, Tone::Neutral, &text, action.as_slice()) == Some(0) {
            context
                .commands
                .send(MaintenanceCommand::InstallGarudaUpdate);
        }

        ui.add_space(ui.tokens().sizes.card_gap);
    }

    card(ui, |ui| {
        for (index, repair) in programs.repairs.iter().enumerate() {
            card_row(ui, index == 0, |ui| {
                if repair_row(ui, context, repair) {
                    if asks_first(&repair.repair) {
                        *asking = Some(repair.repair.clone());
                    } else {
                        run(context, &repair.repair);
                    }
                }
            });
        }
    });

    if let Some(repair) = asking.clone() {
        ask(&ctx, context, &repair, asking);
    }
}

/// Returns whether its Run was clicked.
fn repair_row(ui: &mut Ui, context: &PageContext<'_>, repair: &RepairRow) -> bool {
    let id = format!("maintenance-repair-{}", repair.repair);
    let detail = i18n::text(ui.ctx(), &format!("{id}-detail"));
    let mut clicked = false;
    row(ui, repair_icon(&repair.repair), &id, &detail, |ui| {
        // A greyed Run says what brings it.
        if !repair.available {
            let needs = i18n::text_with(
                ui.ctx(),
                "maintenance-repair-needs",
                &[("package", repair.needs)],
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

/// The question before the task `repair`, which says what is lost. Its
/// name is the Danger button, so the answer repeats what runs. A repair's
/// texts are `maintenance-repair-…`, a Common task's `maintenance-op-…`.
pub(super) fn ask(
    ctx: &egui::Context,
    context: &PageContext<'_>,
    repair: &str,
    asking: &mut Option<String>,
) {
    let id = if repair.starts_with("remote-") {
        format!("maintenance-repair-{repair}")
    } else {
        format!("maintenance-op-{repair}")
    };

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

    let asked = question(ctx, Id::unique("toolbox_gui::maintenance::repair"), texts);
    if asked.confirmed {
        run(context, repair);
    }

    if asked.done {
        *asking = None;
    }
}

fn run(context: &PageContext<'_>, repair: &str) {
    context
        .commands
        .send(MaintenanceCommand::RunNow(repair.to_owned()));
}

fn repair_icon(repair: &str) -> Icon {
    match repair {
        "health-check" => icons::VIEW_TASK,
        "health-fix" => icons::GAMES_HINT,
        "remote-fix" => icons::CONFIGURE,
        "remote-keyring" => icons::SECURITY_HIGH,
        "remote-full-fix" => icons::RUN_BUILD,
        "remote-reset-audio" => icons::AUDIO_VOLUME_HIGH,
        "remote-reset-snapper" => icons::DOCUMENT_REVERT,
        _ => icons::RUN_INSTALL,
    }
}
