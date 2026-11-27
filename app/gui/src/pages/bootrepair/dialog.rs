//! The question before a repair: what is written where, and for a locked
//! system its passphrase, which lives only here until it is sent.

use std::fmt;

use egui::{Context, Id, Label, TextBuffer, Ui};
use toolbox_core::features::bootrepair::page::Offer;
use toolbox_core::features::bootrepair::{
    Encryption, RepairFacts, System, Target, page, rebuild_esp,
};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{BODY, Weight};
use toolbox_gui_kit::widgets::{
    ButtonKind, Confirmation, DialogWidth, ModalWrapper, PropertyTexts, SecretText, SecretTexts,
    Tone, banner, confirm_buttons_why, forget_text, property_list, rich, secret_field,
};
use toolbox_plan::{MAX_PASSWORD_BYTES, Password, RootLayout, VolumeDevice};

use super::texts;
use crate::bootrepair::BootRepair;
use crate::command::BootRepairCommand;
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::PageContext;

pub struct Dialog {
    device: String,
    /// Where GRUB goes, `None` for a new boot menu only.
    target: Option<Target>,
    layout: Option<RootLayout>,
    passphrase: SecretText,
    /// Checked when the field changes, as each check copies the text.
    passphrase_ok: bool,
    field: Option<Id>,
}

/// The list of what is written where, and the sentence below it.
struct Said {
    writes: String,
    disk: String,
    then: String,
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
            passphrase: SecretText::with_room(MAX_PASSWORD_BYTES),
            passphrase_ok: false,
            field: None,
        }
    }

    /// Runs before the dialog is dropped, so the passphrase does not
    /// outlive it in egui's memory.
    pub fn forget(&mut self, ctx: &Context) {
        if let Some(id) = self.field.take() {
            forget_text(ctx, id);
        }

        TextBuffer::clear(&mut self.passphrase);
        self.passphrase_ok = false;
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
        let ready = !context.busy && (!locked || self.passphrase_ok);
        let why = if context.busy {
            Some(i18n::text(ctx, "repair-why-busy"))
        } else if locked && !self.passphrase_ok {
            Some(i18n::text(ctx, "repair-why-passphrase"))
        } else {
            None
        };

        let outcome = ModalWrapper::new(Id::unique("toolbox_gui::bootrepair::dialog"), &title)
            .title_bar(&close)
            .width(DialogWidth::Medium)
            .show(ctx, |ui| {
                // Outside the live ISO nobody vouches for the disk.
                if context.service::<BootRepair>().offer() != Offer::Live {
                    banner(
                        ui,
                        Tone::Warning,
                        &i18n::text(ctx, "repair-trust-disk"),
                        &[],
                    );
                    ui.add_space(ui.tokens().sizes.card_gap);
                }

                self.body(ui, facts, system);
                confirm_buttons_why(ui, &cancel, &confirm, ButtonKind::Danger, why.as_deref())
            });

        if ready && outcome.inner == Some(Confirmation::Confirmed) {
            let passphrase = locked
                .then(|| Password::new(self.passphrase.as_str().to_owned()).ok())
                .flatten();

            self.send(context, system, passphrase);
        }

        outcome.closed || outcome.inner.is_some()
    }

    fn body(&mut self, ui: &mut Ui, facts: &RepairFacts, system: &System) {
        let ctx = ui.ctx().clone();
        let said = self.said(&ctx, facts, system);
        let lies_in = page::layout(system, self.layout).map(|layout| texts::layout(&ctx, layout));
        let text = |id: &str| i18n::text(&ctx, id);
        let (writes, disk, name, lies) = (
            text("repair-fact-writes"),
            text("repair-fact-disk"),
            text("repair-fact-system"),
            text("repair-fact-layout"),
        );

        // No value of the list is copied, so its copy words never show.
        let no_copy = |_: &str| String::new();
        let words = PropertyTexts {
            copy: &no_copy,
            copied: "",
        };

        // The list sets its own row gap, which must not reach the
        // sentences below.
        ui.scope(|ui| {
            property_list(ui, words, |list| {
                list.row(&writes, &said.writes);
                list.row(&disk, &said.disk);
                list.row(&name, &self.device);
                if let Some(layout) = &lies_in {
                    list.row(&lies, layout);
                }
            });
        });

        ui.add_space(ui.tokens().sizes.card_gap);
        ui.add(Label::new(said.then).wrap());
        ui.add_space(ui.tokens().sizes.card_gap);
        ui.add(Label::new(i18n::text(&ctx, "repair-runs-now")).wrap());
        if system.encryption == Encryption::Locked {
            ui.add_space(ui.tokens().sizes.card_gap);
            let note = i18n::text_with(
                &ctx,
                "repair-encrypted",
                &[("system", Arg::from(&self.device))],
            );

            ui.add(Label::new(note).wrap());

            // A name above the field, as the other dialogs' fields have,
            // because a placeholder goes once the first letter is typed.
            let label = i18n::text(&ctx, "repair-passphrase").into_owned();
            ui.add_space(ui.tokens().sizes.card_gap);
            let ink = ui.tokens().text;
            ui.label(rich(ui, &label, Weight::SemiBold, BODY, ink));
            let (show, hide) = (text("password-show"), text("password-hide"));
            let secret = SecretTexts {
                name: &label,
                show: &show,
                hide: &hide,
            };

            let field = secret_field(ui, &mut self.passphrase, secret);

            if field.changed() {
                self.passphrase_ok = Password::new(self.passphrase.as_str().to_owned()).is_ok();
            }

            self.field = Some(field.id);
            ui.ctx()
                .accesskit_node_builder(field.id, |node| node.set_label(label));
        }
    }

    /// The disk is the one written to: the system's own for its boot
    /// files, else the one that takes the boot loader.
    fn said(&self, ctx: &Context, facts: &RepairFacts, system: &System) -> Said {
        let text = |id: &str, words: &[(&'static str, Arg<'_>)]| {
            i18n::text_with(ctx, id, words).into_owned()
        };

        let rebuilt = VolumeDevice::new(self.device.as_str())
            .ok()
            .and_then(|device| rebuild_esp(facts, &device));

        match (&self.target, rebuilt) {
            (None, Some(esp)) => Said {
                writes: text(
                    "repair-writes-regenerate-uefi",
                    &[("esp", Arg::from(esp.as_str()))],
                ),
                disk: texts::disks(system),
                then: text("repair-then-regenerate-uefi", &[]),
            },
            (None, None) => Said {
                writes: text("repair-writes-regenerate", &[]),
                disk: texts::disks(system),
                then: text("repair-then-regenerate", &[]),
            },
            (Some(Target::Esp(esp)), _) => Said {
                writes: text("repair-writes-uefi", &[("target", Arg::from(esp.as_str()))]),
                disk: facts
                    .inventory
                    .esps
                    .iter()
                    .find(|each| each.device == *esp)
                    .map(|each| each.disk.to_string())
                    .unwrap_or_default(),
                then: text("repair-then-uefi", &[]),
            },
            (Some(Target::Disk(disk)), _) => Said {
                writes: text("repair-writes-bios", &[]),
                disk: disk.as_str().to_owned(),
                then: text("repair-then-bios", &[]),
            },
        }
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
