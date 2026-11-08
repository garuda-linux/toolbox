//! The question before a repair: what is written where, and for a locked
//! system its passphrase, which lives only here until it is sent.

use std::fmt;

use egui::{Context, Id, Label, TextEdit, Ui};
use toolbox_core::features::bootrepair::{Encryption, RepairFacts, System, Target, page};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{BODY, Weight};
use toolbox_gui_kit::widgets::{
    ButtonKind, Confirmation, DialogWidth, ModalWrapper, confirm_buttons_why, rich, text_field,
};
use toolbox_plan::{Password, RootLayout, VolumeDevice};

use super::texts;
use crate::command::BootRepairCommand;
use crate::i18n;
use crate::pages::PageContext;

pub struct Dialog {
    device: String,
    /// Where GRUB goes, `None` for a new boot menu only.
    target: Option<Target>,
    layout: Option<RootLayout>,
    passphrase: String,
}

impl fmt::Debug for Dialog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dialog")
            .field("device", &self.device)
            .field("target", &self.target)
            .finish_non_exhaustive()
    }
}

impl Dialog {
    pub fn new(system: &System, target: Option<Target>, layout: Option<RootLayout>) -> Self {
        Self {
            device: system.device().to_owned(),
            target,
            layout,
            passphrase: String::new(),
        }
    }

    pub fn show(&mut self, ctx: &Context, context: &PageContext<'_>, facts: &RepairFacts) -> bool {
        let Some(system) = facts.inventory.system(&self.device) else {
            return true;
        };

        let reinstall = self.target.is_some();
        let (title, confirm) = if reinstall {
            ("repair-reinstall-title", "repair-reinstall-confirm")
        } else {
            ("repair-regenerate-title", "repair-regenerate-confirm")
        };

        let title = i18n::text(ctx, title);
        let confirm = i18n::text(ctx, confirm);
        let cancel = i18n::text(ctx, "repair-cancel");
        let close = i18n::text(ctx, "dialog-close");
        let locked = system.encryption == Encryption::Locked;
        let passphrase = Password::new(self.passphrase.clone()).ok();
        let ready = !context.busy && (!locked || passphrase.is_some());
        let why = if context.busy {
            Some(i18n::text(ctx, "repair-why-busy"))
        } else if locked && passphrase.is_none() {
            Some(i18n::text(ctx, "repair-why-passphrase"))
        } else {
            None
        };

        let outcome = ModalWrapper::new(Id::unique("toolbox_gui::bootrepair::dialog"), &title)
            .title_bar(&close)
            .width(DialogWidth::Medium)
            .show(ctx, |ui| {
                self.body(ui, facts, system);
                confirm_buttons_why(ui, &cancel, &confirm, ButtonKind::Danger, why.as_deref())
            });

        if ready && outcome.inner == Some(Confirmation::Confirmed) {
            self.send(context, system, passphrase.filter(|_| locked));
        }

        outcome.closed || outcome.inner.is_some()
    }

    fn body(&mut self, ui: &mut Ui, facts: &RepairFacts, system: &System) {
        let ctx = ui.ctx().clone();
        ui.add(Label::new(self.does(&ctx, facts, system)).wrap());
        ui.add_space(ui.tokens().sizes.card_gap);
        if let Some(layout) = page::layout(system, self.layout) {
            let layout = texts::layout(&ctx, layout);
            ui.add(
                Label::new(i18n::text_with(
                    &ctx,
                    "repair-looked-for",
                    &[("layout", &layout)],
                ))
                .wrap(),
            );
            ui.add_space(ui.tokens().sizes.card_gap);
        }

        ui.add(Label::new(i18n::text(&ctx, "repair-runs-now")).wrap());
        if system.encryption == Encryption::Locked {
            ui.add_space(ui.tokens().sizes.card_gap);
            let note = i18n::text_with(&ctx, "repair-encrypted", &[("system", &self.device)]);
            ui.add(Label::new(note).wrap());

            // A name above the field, as the other dialogs' fields have,
            // because a placeholder goes once the first letter is typed.
            let name = i18n::text(&ctx, "repair-passphrase").into_owned();
            ui.add_space(ui.tokens().sizes.card_gap);
            let text = ui.tokens().text;
            ui.label(rich(ui, &name, Weight::SemiBold, BODY, text));
            let field = text_field(
                ui,
                TextEdit::singleline(&mut self.passphrase)
                    .password(true)
                    .desired_width(f32::INFINITY)
                    .align(egui::Align2::LEFT_CENTER),
            );

            ui.ctx()
                .accesskit_node_builder(field.id, |node| node.set_label(name));
        }
    }

    fn does(&self, ctx: &Context, facts: &RepairFacts, system: &System) -> String {
        let disks = texts::disks(system);
        let text = match &self.target {
            None => i18n::text_with(
                ctx,
                "repair-does-regenerate",
                &[("system", &self.device), ("disks", &disks)],
            ),
            Some(Target::Esp(esp)) => {
                let disk = facts
                    .inventory
                    .esps
                    .iter()
                    .find(|each| each.device == *esp)
                    .map(|each| each.disk.to_string())
                    .unwrap_or_default();

                let words = [
                    ("target", esp.as_str()),
                    ("disk", disk.as_str()),
                    ("system", &self.device),
                    ("disks", &disks),
                ];

                i18n::text_with(ctx, "repair-does-uefi", &words)
            }
            Some(Target::Disk(disk)) => {
                let words = [
                    ("target", disk.as_str()),
                    ("system", &self.device),
                    ("disks", &disks),
                ];

                i18n::text_with(ctx, "repair-does-bios", &words)
            }
        };

        text.into_owned()
    }

    fn send(&self, context: &PageContext<'_>, system: &System, passphrase: Option<Password>) {
        let Ok(device) = VolumeDevice::new(system.device()) else {
            return;
        };

        context.commands.send(BootRepairCommand::Repair {
            system: device,
            layout: page::layout(system, self.layout),
            passphrase,
            target: self.target.clone(),
        });
    }
}
