//! Each is a card of setting rows. Every change waits in the queue, and
//! Apply writes `/etc/default/grub` and builds the menu again.

mod kernel;
mod look;
mod menu;
#[cfg(test)]
mod tests;

use egui::Ui;
use toolbox_core::features::boot::Flag;
use toolbox_core::features::boot::edit::Edit;
use toolbox_core::features::boot::view::{self, Page, Probed, Readiness};
use toolbox_gui_kit::widgets::{ButtonKind, Tone, banner, empty_state, pill_button};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_system::{FailedProbes, Host, Menu, Probe};

use super::header::refresh_header;
use super::rows::refusal_banner;
use super::{PageContext, PageView, plan_running};
use crate::command::BootCommand;
use crate::i18n;
use crate::pages::heading;

/// The width of a row's control: a menu entry's name fits.
const WIDE_CONTROL: f32 = toolbox_gui_kit::tokens::Sizes::DEFAULT.setting_control_wide;

#[derive(Debug, Default)]
pub struct BootPage {
    entries: menu::EntryRows,
    parameter: String,
    /// The typed parameter was sent last frame. It stays in the field when
    /// it was refused, so it can be corrected.
    sent: Option<String>,
}

impl PageView for BootPage {
    fn header(&mut self, ui: &mut Ui, context: &PageContext<'_>) -> bool {
        if refresh_header(ui, "page-boot", "page-boot-subtitle", "boot-refresh") {
            context.commands.send(BootCommand::Refresh);
        }

        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = tracing::trace_span!("boot ui").entered();
        let ctx = ui.ctx().clone();
        refusal(ui, context);

        let readiness = view::state(context.snapshot, probed(context));
        let (icon, title, detail) = match readiness {
            Readiness::Ready => {
                self.ready(ui, context);
                return;
            }
            Readiness::Failed => (icons::DIALOG_ERROR, "boot-failed", "boot-failed-detail"),
            Readiness::Missing => (icons::HELP_ABOUT, "boot-missing", "boot-missing-detail"),
            Readiness::Loading => (icons::DRIVE_HARDDISK_ROOT, "boot-loading", ""),
        };

        let detail = if detail.is_empty() {
            String::new()
        } else {
            i18n::text(&ctx, detail).into_owned()
        };

        empty_state(ui, icon, &i18n::text(&ctx, title), &detail);
    }
}

impl BootPage {
    fn ready(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let ctx = ui.ctx().clone();
        plan_running(ui, context);

        let page = Page::read(context.snapshot, context.changes);
        let splash = page.flag(Flag::Splash);
        if view::virtual_machine(context.snapshot) && splash {
            banner(ui, Tone::Neutral, &i18n::text(&ctx, "boot-vm"), &[]);
            ui.add_space(ui.tokens().sizes.card_gap);
        }

        proposal(ui, context, &page);
        ui.add_enabled_ui(!context.busy, |ui| {
            menu::show(ui, context, &page, &mut self.entries);
            ui.add_space(ui.tokens().sizes.section_gap);
            self.kernel(ui, context, &page);
            ui.add_space(ui.tokens().sizes.section_gap);
            look::show(ui, context, &page);
        });
    }

    /// The kernel card, which clears a typed parameter once it was taken.
    fn kernel(&mut self, ui: &mut Ui, context: &PageContext<'_>, page: &Page) {
        if let Some(sent) = self.sent.take() {
            let refused = context
                .boot
                .refusal()
                .is_some_and(|refusal| refusal.value == sent);

            if !refused && self.parameter == sent {
                self.parameter.clear();
            }
        }

        if let Some(added) = kernel::show(ui, context, page, &mut self.parameter) {
            self.sent = Some(added.clone());
            send(context, Edit::AddParameter(added));
        }
    }
}

/// Why the last edit was not queued, until dismissed.
fn refusal(ui: &mut Ui, context: &PageContext<'_>) {
    let Some(refusal) = context.boot.refusal() else {
        return;
    };

    let id = format!("boot-refused-{}", refusal.code);
    let text = i18n::text_with(ui.ctx(), &id, &[("value", &refusal.value)]);
    if refusal_banner(ui, Tone::Error, &text, "boot-dismiss") {
        context.commands.send(BootCommand::DismissRefusal);
    }
}

/// A kernel the Kernels page proposed as the default, until answered.
fn proposal(ui: &mut Ui, context: &PageContext<'_>, page: &Page) {
    let Some(kernel) = context.boot.proposal() else {
        return;
    };

    let ctx = ui.ctx().clone();
    let menu = context.boot.menu();
    let entry = menu.kernel_entry(kernel);
    let current = page.entry_value(menu.known());
    let state = if matches!(menu, Menu::NotAsked | Menu::Loading) {
        "loading"
    } else if entry.is_empty() {
        "none"
    } else if page.remember() {
        "remember"
    } else if entry == current {
        "same"
    } else {
        "offer"
    };

    let text = i18n::text_with(
        &ctx,
        &format!("boot-proposal-{state}"),
        &[("kernel", kernel.as_str())],
    );

    banner(ui, Tone::Accent, &text, &[]);
    ui.horizontal(|ui| {
        if state == "offer" {
            let queue = i18n::text(&ctx, "boot-proposal-queue");
            if pill_button(ui, &queue, ButtonKind::Primary).clicked() {
                send(context, Edit::Default(entry.to_owned()));
                context.commands.send(BootCommand::DropProposal);
            }
        }

        let close = if state == "offer" {
            "boot-proposal-keep"
        } else {
            "boot-proposal-close"
        };

        if pill_button(ui, &i18n::text(&ctx, close), ButtonKind::Soft).clicked() {
            context.commands.send(BootCommand::DropProposal);
        }
    });

    ui.add_space(ui.tokens().sizes.card_gap);
}

fn probed(context: &PageContext<'_>) -> Probed {
    Probed {
        grub_failed: context
            .snapshot
            .part::<FailedProbes>()
            .is_some_and(|failed| failed.contains(Probe::Grub)),
        read: context.snapshot.part::<Host>().is_some(),
    }
}

fn send(context: &PageContext<'_>, change: Edit) {
    context.commands.send(BootCommand::Edit(change));
}

fn queued(ctx: &egui::Context, queued: bool) -> Option<String> {
    queued.then(|| i18n::text(ctx, "boot-queued").into_owned())
}
