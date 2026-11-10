//! The Boot Repair page: installs GRUB for an installed system again or
//! writes its boot menu again, from the live system.
//!
//! The page only shows on the live ISO, or with `--boot-repair`. Each
//! repair asks first, naming the partition and the disk it writes, and
//! runs at once in the Console.

mod dialog;
mod system;
#[cfg(test)]
mod tests;
mod texts;

use egui::{Label, Ui};
use toolbox_core::features::bootrepair::Target;
use toolbox_core::features::bootrepair::page::{self, Offer, Progress, State};
use toolbox_gui_kit::widgets::{ButtonKind, Tone, banner, empty_state, pill_button};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_plan::RootLayout;

use self::dialog::Dialog;
use super::header::refresh_header;
use super::{PageContext, PageView};
use crate::bootrepair::BootRepair;
use crate::command::BootRepairCommand;
use crate::i18n;

#[derive(Debug, Default)]
pub struct BootRepairPage {
    /// The device of the picked system, `None` for the usual one.
    system: Option<String>,
    target: Option<Target>,
    layout: Option<RootLayout>,
    dialog: Option<Dialog>,
}

impl PageView for BootRepairPage {
    /// The question may hold a passphrase, which must not wait for the return.
    fn left(&mut self, ctx: &egui::Context) {
        if let Some(mut dialog) = self.dialog.take() {
            dialog.forget(ctx);
        }
    }

    fn header(&mut self, ui: &mut Ui, context: &PageContext<'_>) -> bool {
        let (title, subtitle) = ("page-boot-repair", "page-boot-repair-subtitle");
        if refresh_header(ui, title, subtitle, "repair-refresh") {
            context.commands.send(BootRepairCommand::Refresh);
        }

        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = tracing::trace_span!("boot repair ui").entered();
        let ctx = ui.ctx().clone();
        let text = |id: &str| i18n::text(&ctx, id).into_owned();
        let repair = context.service::<BootRepair>();
        notes(ui, context);

        let state = page::state(repair.offer(), repair.facts());
        let facts = repair.facts().known(repair.offer());
        let icon = icons::RUN_BUILD_CONFIGURE;
        let Some(facts) = facts.filter(|_| state == State::Ready) else {
            match state {
                State::Hidden => empty_state(ui, icon, &text("repair-hidden"), ""),
                State::Loading => empty_state(ui, icon, &text("repair-reading"), ""),
                State::Failed => empty_state(
                    ui,
                    icon,
                    &text("repair-failed"),
                    &text("repair-failed-detail"),
                ),
                State::Missing | State::Ready => {
                    let tools = facts
                        .map(|facts| {
                            let names: Vec<&str> =
                                facts.missing.iter().map(|tool| tool.name()).collect();

                            names.join(", ")
                        })
                        .unwrap_or_default();

                    let detail =
                        i18n::text_with(&ctx, "repair-missing-detail", &[("tools", &tools)]);

                    empty_state(ui, icon, &text("repair-missing"), &detail)
                }
            };

            return;
        };

        let Some(system) = page::system(facts, self.system.as_deref()) else {
            empty_state(ui, icon, &text("repair-none"), &text("repair-none-detail"));
            return;
        };

        let picks = system::show(ui, context, (facts, system), self);
        if let Some(dialog) = picks {
            self.dialog = Some(dialog);
        }

        if let Some(dialog) = &mut self.dialog
            && dialog.show(&ctx, context, facts)
        {
            dialog.forget(&ctx);
            self.dialog = None;
        }
    }
}

fn notes(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let repair = context.service::<BootRepair>();
    if repair.offer() == Offer::Test {
        banner(ui, Tone::Warning, &text("repair-test-mode"), &[]);
        ui.add_space(ui.tokens().sizes.card_gap);
    }

    ui.add(Label::new(text("repair-intro")).wrap());
    let dismiss = text("repair-dismiss");
    if let Some(code) = context.refusals.boot_repair {
        let refused = texts::refusal(&ctx, code);
        ui.add_space(ui.tokens().sizes.card_gap);
        if banner(ui, Tone::Error, &refused, &[&dismiss]) == Some(0) {
            context.commands.send(BootRepairCommand::DismissRefusal);
        }
    }

    progress(ui, context, &dismiss);
    ui.add_space(ui.tokens().sizes.section_gap);
}

fn progress(ui: &mut Ui, context: &PageContext<'_>, dismiss: &str) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let progress = context.service::<BootRepair>().progress();
    let Some(said) = texts::progress(&ctx, progress) else {
        return;
    };

    ui.add_space(ui.tokens().sizes.card_gap);

    let (tone, action) = match progress {
        Progress::Succeeded => (Tone::Success, Some(dismiss)),
        Progress::Failed => (Tone::Error, Some(dismiss)),
        Progress::Busy => (Tone::Warning, Some(dismiss)),
        Progress::None | Progress::Waiting | Progress::Running => (Tone::Neutral, None),
    };

    if banner(ui, tone, &said, action.as_slice()) == Some(0) {
        context.commands.send(BootRepairCommand::Forget);
    }

    if !context.console_open
        && pill_button(ui, &text("repair-show-console"), ButtonKind::Soft).clicked()
    {
        context.commands.send(BootRepairCommand::ShowConsole);
    }
}
