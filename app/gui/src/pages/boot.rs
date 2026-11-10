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
    ButtonKind, DialogWidth, Question, Tone, banner, empty_state, pill_button, question,
};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_plan::PackageName;
use toolbox_system::{FailedProbes, Host, Menu, Probe};

use super::header::refresh_header;
use super::rows::refusal_banner;
use super::{PageContext, PageView, plan_running};
use crate::boot::Boot;
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
        let subtitle = if sdboot(context) {
            "page-boot-subtitle-sdboot"
        } else {
            "page-boot-subtitle"
        };

        if refresh_header(ui, "page-boot", subtitle, "boot-refresh") {
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
        plan_running(ui, context);

        let page = Page::read(context.snapshot, context.changes);

        vm_banner(ui, context, page.flag(Flag::Splash));
        proposal(ui, context, &page);
        ui.add_enabled_ui(!context.busy, |ui| {
            menu::show(ui, context, &page, &mut self.entries);
            ui.add_space(ui.tokens().sizes.section_gap);
            self.kernel(ui, context, &page);
            ui.add_space(ui.tokens().sizes.section_gap);
            look::show(ui, context, &page);
        });
    }

    fn sdboot(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let gap = ui.tokens().sizes.section_gap;

        refusal(ui, context);
        plan_running(ui, context);

        let page = Page::read(context.snapshot, context.changes);
        let splash = page.flag(Flag::Splash);

        vm_banner(ui, context, splash);
        sdboot::proposal(ui, context);
        ui.add_enabled_ui(!context.busy, |ui| {
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
                .is_some_and(|refusal| refusal.value == sent);

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
}

/// The splash works in a limited way in a virtual machine.
fn vm_banner(ui: &mut Ui, context: &PageContext<'_>, splash: bool) {
    if view::virtual_machine(context.snapshot) && splash {
        banner(ui, Tone::Neutral, &i18n::text(ui.ctx(), "boot-vm"), &[]);
        ui.add_space(ui.tokens().sizes.card_gap);
    }
}

/// Why the last edit was not queued, until dismissed.
fn refusal(ui: &mut Ui, context: &PageContext<'_>) {
    let Some(refusal) = context.service::<Boot>().refusal() else {
        return;
    };

    if refusal.code == "needs-confirm" {
        risky(ui.ctx(), context, &refusal.value);
        return;
    }

    let id = format!("boot-refused-{}", refusal.code);
    let text = i18n::text_with(ui.ctx(), &id, &[("value", &refusal.value)]);
    if refusal_banner(ui, Tone::Error, &text, "boot-dismiss") {
        context.commands.send(BootCommand::DismissRefusal);
    }
}

/// The question before a parameter that opens a root shell at boot or
/// switches a security layer off.
fn risky(ctx: &egui::Context, context: &PageContext<'_>, value: &str) {
    let title = i18n::text_with(ctx, "boot-risky-title", &[("value", value)]);
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
        (false, true) => context.commands.send(BootCommand::DismissRefusal),
        (false, false) => {}
    }
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
        &[("kernel", kernel.as_str())],
    );

    banner(ui, Tone::Accent, &text, &[]);
    ui.horizontal(|ui| {
        if state == "offer" {
            let label = i18n::text(&ctx, "boot-proposal-queue");
            if pill_button(ui, &label, ButtonKind::Primary).clicked() {
                queue();
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

fn sdboot(context: &PageContext<'_>) -> bool {
    context.snapshot.part::<BootLoader>() == Some(&BootLoader::SystemdBoot)
}
