//! The Languages page's clicks, which all edit the queue, and the last one
//! it refused.

#[cfg(test)]
mod tests;

use toolbox_core::features::locales::view::{add_missing_packs, toggle_pack};
use toolbox_core::features::locales::{self as core_locales, choose_system};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::Toggle;
use toolbox_services::queue::edit::QueueEdit;

use crate::command::{AppCommand, LocaleCommand};
use crate::page_service::Handled;
use crate::refused::{Refused, queue_or_refuse};

pub type LocaleRefusal = Refused<core_locales::Refusal>;

#[derive(Debug, Default)]
pub struct Locales {
    refusal: Option<LocaleRefusal>,
}

impl Locales {
    pub const fn refusal(&self) -> Option<&LocaleRefusal> {
        self.refusal.as_ref()
    }

    pub fn set_refusal(&mut self, refusal: Option<LocaleRefusal>) {
        self.refusal = refusal;
    }

    pub fn handle(
        &mut self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        let AppCommand::Locale(command) = command else {
            return Handled::default();
        };

        match command {
            LocaleCommand::Toggle { entry, on } => {
                let subject = entry.locale.as_str().to_owned();
                queue_or_refuse(&mut self.refusal, subject, changes, |changes| {
                    core_locales::toggle(entry, Toggle::from_on(*on), changes, snapshot)
                })
            }
            LocaleCommand::System(locale) => {
                let subject = locale.as_str().to_owned();
                queue_or_refuse(&mut self.refusal, subject, changes, |changes| {
                    choose_system(locale.clone(), changes, snapshot)
                })
            }
            LocaleCommand::TogglePack { package, on } => {
                let (edit, listed) = QueueEdit::of_click(changes, |changes| {
                    toggle_pack(package, Toggle::from_on(*on), changes, snapshot)
                });

                if !listed {
                    tracing::warn!("{package} is no listed language pack, so it was not queued");
                }

                Handled::edit(edit)
            }
            LocaleCommand::AddMissingPacks => {
                let (edit, added) =
                    QueueEdit::of_click(changes, |changes| add_missing_packs(changes, snapshot));

                tracing::debug!("queued {added} missing language packs");
                Handled::edit(edit)
            }
            LocaleCommand::DismissRefusal => {
                self.refusal = None;
                Handled::taken()
            }
            LocaleCommand::Refresh => Handled::answers(vec![AppCommand::ReadSystem]),
        }
    }
}
