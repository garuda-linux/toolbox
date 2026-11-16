//! Each is a card of setting rows. Every change waits in the queue, and
//! Apply writes `/etc/default/grub` and builds the menu again.

mod kernel;
mod look;
mod menu;
mod sdboot;
#[cfg(test)]
mod tests;

use egui::Ui;
use toolbox_core::domains::BootLoader;
use toolbox_core::features::boot::edit::Edit;
use toolbox_core::features::boot::view::{self, Page, Probed, Readiness};
use toolbox_core::features::boot::{Flag, kernel};
use toolbox_gui_kit::widgets::{
    ButtonKind, DialogWidth, Question, Tone, banner, card, empty_state, empty_state_action,
    question, row_height, skeleton_rows, text_block_height,
};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_plan::PackageName;
use toolbox_services::boot_menu::Menu;
use toolbox_system::{FailedProbes, Host, Probe};

use super::header::refresh_header_reading;
use super::rows::refused_banner;
use super::{PageContext, PageView, plan_running};
use crate::boot::Boot;
use crate::command::{AppCommand, BootCommand};
use crate::i18n;
use crate::i18n::Arg;
use crate::page::PageId;
use crate::pages::heading;

/// Placeholder rows while the system is read, as many as the menu card has.
const SKELETON_ROWS: usize = 3;

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
        let subtitle = if sdboot(context) {
            "page-boot-subtitle-sdboot"
        } else {
            "page-boot-subtitle"
        };

        let reading = context.service::<Boot>().reading();
        let titles = ("page-boot", subtitle);
        if refresh_header_reading(ui, titles, "boot-refresh", reading) {
            context.commands.send(BootCommand::Refresh);
        }

        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = tracing::trace_span!("boot ui").entered();
        let ctx = ui.ctx().clone();

        if sdboot(context) {
            self.sdboot(ui, context);
            return;
        }

        self.refusal(ui, context);

        let text = |id: &str| i18n::text(&ctx, id).into_owned();
        match view::state(context.snapshot, probed(context)) {
            Readiness::Ready => self.ready(ui, context),
            Readiness::Failed => {
                let (title, detail) = (text("boot-failed"), text("boot-failed-detail"));
                let again = text("common-read-again");
                if empty_state_action(ui, icons::DIALOG_ERROR, &title, &detail, &again).clicked() {
                    context.commands.send(BootCommand::Refresh);
                }
            }
            Readiness::Missing => {
                let (title, detail) = (text("boot-missing"), text("boot-missing-detail"));
                _ = empty_state(ui, icons::HELP_ABOUT, &title, &detail);
            }
            Readiness::Loading => {
                let height = row_height(ui, text_block_height(ui, 1));
                let name = text("boot-loading");
                card(ui, |ui| skeleton_rows(ui, SKELETON_ROWS, height, &name));
            }
        }
    }
}

impl BootPage {
    fn ready(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        plan_running(ui, context);

        let page = Page::read(context.snapshot, context.changes);

        vm_banner(ui, context, page.flag(Flag::Splash));
        proposal(ui, context, &page);
        ui.enabled_ui(!context.busy, |ui| {
            menu::show(ui, context, &page, &mut self.entries);
            ui.add_space(ui.tokens().sizes.section_gap);
            self.kernel(ui, context, &page);
            ui.add_space(ui.tokens().sizes.section_gap);
            look::show(ui, context, &page);
        });
    }

    fn sdboot(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let gap = ui.tokens().sizes.section_gap;

        self.refusal(ui, context);
        plan_running(ui, context);

        let page = Page::read(context.snapshot, context.changes);
        let splash = page.flag(Flag::Splash);

        vm_banner(ui, context, splash);
        sdboot::proposal(ui, context);
        ui.enabled_ui(!context.busy, |ui| {
            sdboot::menu_card(ui, context);
            ui.add_space(gap);

            if kernel(context.snapshot).is_some() {
                self.kernel(ui, context, &page);
            } else {
                sdboot::no_cmdline_card(ui);
            }

            if splash {
                ui.add_space(gap);
                look::plymouth_card(ui, context);
            }
        });
    }

    /// The kernel card, which clears a typed parameter once it was taken.
    fn kernel(&mut self, ui: &mut Ui, context: &PageContext<'_>, page: &Page) {
        if let Some(sent) = self.sent.take() {
            let refused = context
                .service::<Boot>()
                .refusal()
                .is_some_and(|refusal| refusal.subject == sent);

            if !refused && self.parameter == sent {
                self.parameter.clear();
            }
        }

        if let Some(added) = kernel::show(ui, context, page, &mut self.parameter) {
            self.sent = Some(added.clone());
            send(
                context,
                Edit::AddParameter {
                    text: added,
                    confirmed: false,
                },
            );
        }
    }

    /// The refusal frame used up `sent` already, so a confirmed risky
    /// parameter clears the field here.
    fn refusal(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        if refusal(ui, context) {
            self.parameter.clear();
        }
    }
}

/// The splash works in a limited way in a virtual machine.
fn vm_banner(ui: &mut Ui, context: &PageContext<'_>, splash: bool) {
    if view::virtual_machine(context.snapshot) && splash {
        banner(ui, Tone::Neutral, &i18n::text(ui.ctx(), "boot-vm"), &[]);
        ui.add_space(ui.tokens().sizes.card_gap);
    }
}

/// Why the last edit was not queued, until dismissed. Returns whether a
/// risky parameter was confirmed and sent again.
fn refusal(ui: &mut Ui, context: &PageContext<'_>) -> bool {
    let Some(refusal) = context.service::<Boot>().refusal() else {
        return false;
    };

    if refusal.refusal.needs_confirm() {
        return risky(ui.ctx(), context, &refusal.subject);
    }

    if refused_banner(ui, Some(refusal), Tone::Error, "boot-dismiss") {
        context
            .commands
            .send(AppCommand::DismissRefusal(PageId::Boot));
    }

    false
}

/// The question before a parameter that opens a root shell at boot or
/// switches a security layer off. Returns whether it was confirmed.
fn risky(ctx: &egui::Context, context: &PageContext<'_>, value: &str) -> bool {
    let title = i18n::text_with(ctx, "boot-risky-title", &[("value", Arg::from(value))]);
    let body = i18n::text(ctx, "boot-risky-body");
    let close = i18n::text(ctx, "dialog-close");
    let cancel = i18n::text(ctx, "boot-risky-cancel");
    let confirm = i18n::text(ctx, "boot-risky-confirm");
    let texts = Question {
        title: &title,
        close: &close,
        body: &body,
        cancel: &cancel,
        confirm: &confirm,
        kind: ButtonKind::Danger,
        width: DialogWidth::Small,
    };

    let asked = question(ctx, egui::Id::unique("toolbox_gui::boot::risky"), texts);

    match (asked.confirmed, asked.done) {
        (true, _) => send(
            context,
            Edit::AddParameter {
                text: value.to_owned(),
                confirmed: true,
            },
        ),
        (false, true) => context
            .commands
            .send(AppCommand::DismissRefusal(PageId::Boot)),
        (false, false) => {}
    }

    asked.confirmed
}

/// A kernel the Kernels page proposed as the default, until answered.
fn proposal(ui: &mut Ui, context: &PageContext<'_>, page: &Page) {
    let Some(kernel) = context.service::<Boot>().proposal() else {
        return;
    };

    let menu = context.service::<Boot>().menu();
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

    note(ui, context, kernel, state, || {
        send(context, Edit::Default(entry.to_owned()));
    });
}

fn note(
    ui: &mut Ui,
    context: &PageContext<'_>,
    kernel: &PackageName,
    state: &str,
    queue: impl FnOnce(),
) {
    let ctx = ui.ctx().clone();
    let text = i18n::text_with(
        &ctx,
        &format!("boot-proposal-{state}"),
        &[("kernel", Arg::from(kernel.as_str()))],
    );

    let offer = state == "offer";
    let queue_text = i18n::text(&ctx, "boot-proposal-queue");
    let close_text = i18n::text(
        &ctx,
        if offer {
            "boot-proposal-keep"
        } else {
            "boot-proposal-close"
        },
    );

    let actions: &[&str] = if offer {
        &[&*queue_text, &*close_text]
    } else {
        &[&*close_text]
    };

    match banner(ui, Tone::Accent, &text, actions).value {
        Some(0) if offer => {
            queue();
            context.commands.send(BootCommand::DropProposal);
        }
        Some(_) => context.commands.send(BootCommand::DropProposal),
        None => {}
    }

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

fn sdboot(context: &PageContext<'_>) -> bool {
    context.snapshot.part::<BootLoader>() == Some(&BootLoader::SystemdBoot)
}
