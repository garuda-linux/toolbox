//! What the app does with a click on the Languages page. Every click
//! edits the queue, a refused one stays on the page until dismissed or
//! the next click is taken.

use toolbox_core::features::locales::view::{add_missing_packs, toggle_pack};
use toolbox_core::features::locales::{Refusal, choose_system, toggle};
use toolbox_plan::Toggle;

use super::App;
use crate::command::LocaleCommand;
use crate::pages::LocaleRefusal;

impl App {
    pub(super) fn run_locale(&mut self, command: LocaleCommand) {
        let snapshot = self.current_snapshot();
        match command {
            LocaleCommand::Toggle { entry, on } => {
                let result = self.queue.edit(&snapshot, |changes, snapshot| {
                    toggle(&entry, Toggle::from_on(on), changes, snapshot)
                });

                self.refuse_locale(result, entry.locale.as_str());
            }
            LocaleCommand::System(locale) => {
                let name = locale.as_str().to_owned();
                let result = self.queue.edit(&snapshot, |changes, snapshot| {
                    choose_system(locale, changes, snapshot)
                });

                self.refuse_locale(result, &name);
            }
            LocaleCommand::TogglePack { package, on } => {
                let listed = self.queue.edit(&snapshot, |changes, snapshot| {
                    toggle_pack(&package, Toggle::from_on(on), changes, snapshot)
                });

                if !listed {
                    tracing::warn!("{package} is no listed language pack, so it was not queued");
                }
            }
            LocaleCommand::AddMissingPacks => {
                let added = self.queue.edit(&snapshot, add_missing_packs);
                tracing::debug!("queued {added} missing language packs");
            }
            LocaleCommand::DismissRefusal => self.refusals.locale = None,
            LocaleCommand::Refresh => self.read_system(),
        }
    }

    fn refuse_locale(&mut self, result: Result<(), Refusal>, locale: &str) {
        self.refusals.locale = result.err().map(|refusal| LocaleRefusal {
            code: refusal.as_str(),
            locale: locale.to_owned(),
        });
    }
}
