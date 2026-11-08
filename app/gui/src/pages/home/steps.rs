//! The live ISO's tools.

use egui::Ui;
use toolbox_core::features::maintenance::RunNow;
use toolbox_gui_kit::fonts::{SECTION, Weight};
use toolbox_gui_kit::widgets::{
    ActionRow, ButtonKind, Tone, banner, card, card_row, pill_button, rich,
};
use toolbox_gui_kit::{UiExt as _, icons, tokens};

use super::HomeProblem;
use super::status::run_row;
use crate::command::HomeCommand;
use crate::i18n;
use crate::pages::PageContext;

/// The installer and the repair shell, each where its program exists.
pub fn live(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    heading(ui, "home-live");
    ui.add_space(ui.tokens().sizes.card_gap);

    if let Some(HomeProblem::Installer(reason)) = context.home.problem {
        let text = i18n::text_with(&ctx, "home-install-failed", &[("reason", reason)]);
        banner(ui, Tone::Error, &text, &[]);
        ui.add_space(ui.tokens().sizes.card_gap);
    }

    if context.home.programs_failed {
        let text = i18n::text(&ctx, "home-programs-failed");
        let again = i18n::text(&ctx, "common-read-again");
        if banner(ui, Tone::Warning, &text, &[&again]) == Some(0) {
            context.commands.send(HomeCommand::Refresh);
        }

        ui.add_space(ui.tokens().sizes.card_gap);
    }

    let programs = context.home.programs.cloned().unwrap_or_default();
    if !programs.installer && !programs.chroot {
        return;
    }

    card(ui, |ui| {
        if programs.installer {
            card_row(ui, true, |ui| {
                let title = i18n::text(&ctx, "home-install");
                let detail = i18n::text(&ctx, "home-install-detail");
                ActionRow::new(icons::RUN_INSTALL, Tone::Accent, &title)
                    .detail(&detail)
                    .show(ui, |ui| {
                        let start = i18n::text(&ctx, "home-install-button");
                        if pill_button(ui, &start, ButtonKind::Primary).clicked() {
                            context.commands.send(HomeCommand::StartInstaller);
                        }
                    });
            });
        }

        if programs.chroot {
            card_row(ui, !programs.installer, |ui| {
                run_row(
                    ui,
                    context,
                    (icons::UTILITIES_TERMINAL, Tone::Neutral),
                    ["home-chroot", "home-chroot-detail", "home-chroot-button"],
                    RunNow::Chroot,
                );
            });
        }
    });
}

pub(super) fn heading(ui: &mut Ui, id: &str) {
    let tokens = tokens::current(ui.ctx());
    let text = i18n::text(ui.ctx(), id);
    ui.label(rich(ui, &text, Weight::Bold, SECTION, tokens.text));
}
