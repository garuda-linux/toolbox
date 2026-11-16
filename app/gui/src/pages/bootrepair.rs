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

use egui::Ui;
use toolbox_core::features::bootrepair::Target;
use toolbox_core::features::bootrepair::page::{self, Offer, Progress, State};
use toolbox_gui_kit::widgets::{
    Tone, banner, card, empty_state, empty_state_action, row_height, skeleton_rows,
    text_block_height,
};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_plan::RootLayout;

use self::dialog::Dialog;
use super::header::refresh_header_reading;
use super::{PageContext, PageView, QueueShown};
use crate::bootrepair::BootRepair;
use crate::command::{AppCommand, BootRepairCommand};
use crate::i18n;
use crate::i18n::Arg;
use crate::page::PageId;
use crate::refused::Shown as _;

/// Placeholder rows while the disks are read, as many as the system card has.
const SKELETON_ROWS: usize = 4;

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
        let reading = context.service::<BootRepair>().reading();
        if refresh_header_reading(ui, (title, subtitle), "repair-refresh", reading) {
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
                State::Loading => {
                    let height = row_height(ui, text_block_height(ui, 1));
                    let name = text("repair-reading");
                    card(ui, |ui| skeleton_rows(ui, SKELETON_ROWS, height, &name)).inner
                }
                State::Failed => {
                    let (title, detail) = (text("repair-failed"), text("repair-failed-detail"));
                    let again = text("common-read-again");
                    let error = icons::DIALOG_ERROR;
                    if empty_state_action(ui, error, &title, &detail, &again).clicked() {
                        context.commands.send(BootRepairCommand::Refresh);
                    }

                    return;
                }
                State::Missing | State::Ready => {
                    let tools = facts
                        .map(|facts| {
                            let names: Vec<&str> =
                                facts.missing.iter().map(|tool| tool.name()).collect();

                            names.join(", ")
                        })
                        .unwrap_or_default();

                    let detail = i18n::text_with(
                        &ctx,
                        "repair-missing-detail",
                        &[("tools", Arg::from(&tools))],
                    );

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

    /// Each repair runs at once after its question, so the queue would be a
    /// second way to act here.
    fn queue_shown(&self) -> QueueShown {
        QueueShown::Never
    }
}

/// The banners above the page's content, with room under them only when
/// one shows.
fn notes(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let gap = ui.tokens().sizes.card_gap;
    let repair = context.service::<BootRepair>();
    let mut shown = repair.offer() == Offer::Test;
    if shown {
        banner(
            ui,
            Tone::Warning,
            &i18n::text(&ctx, "repair-test-mode"),
            &[],
        );
    }

    let dismiss = i18n::text(&ctx, "repair-dismiss");
    if let Some(refusal) = repair.refusal() {
        let refused = refusal.text(&ctx);
        if shown {
            ui.add_space(gap);
        }

        if banner(ui, Tone::Error, &refused, &[&dismiss]).value == Some(0) {
            context
                .commands
                .send(AppCommand::DismissRefusal(PageId::BootRepair));
        }

        shown = true;
    }

    if progress(ui, context, &dismiss, shown) || shown {
        ui.add_space(ui.tokens().sizes.section_gap);
    }
}

/// The run's line, with Show Console in it while the Console is hidden.
/// Returns whether it showed.
fn progress(ui: &mut Ui, context: &PageContext<'_>, dismiss: &str, below: bool) -> bool {
    let ctx = ui.ctx().clone();
    let progress = context.service::<BootRepair>().progress();
    let Some(said) = texts::progress(&ctx, progress) else {
        return false;
    };

    if below {
        ui.add_space(ui.tokens().sizes.card_gap);
    }

    let (tone, dismissable) = match progress {
        Progress::Succeeded => (Tone::Success, true),
        Progress::Failed => (Tone::Error, true),
        Progress::Busy => (Tone::Warning, true),
        Progress::None | Progress::Waiting | Progress::Running => (Tone::Neutral, false),
    };

    let console = i18n::text(&ctx, "repair-show-console");
    let mut actions = Vec::new();
    if !context.console_open {
        actions.push(&*console);
    }

    if dismissable {
        actions.push(dismiss);
    }

    let picked = banner(ui, tone, &said, &actions)
        .value
        .and_then(|index| actions.get(index).copied());

    match picked {
        Some(picked) if picked == &*console => {
            context.commands.send(BootRepairCommand::ShowConsole);
        }
        Some(_) => context.commands.send(BootRepairCommand::Forget),
        None => {}
    }

    true
}
