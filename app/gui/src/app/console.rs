//! What the app does with a choice in the Console's menu and with the
//! notes of the output's upload. The link goes to the clipboard as the
//! Diagnostics report's does, by the same setting.

use toolbox_core::features::maintenance::page::upgrader;
use toolbox_core::features::packages::mirror::retry_after_upgrade;
use toolbox_gui_kit::toasts::ToastKind;

use super::App;
use crate::command::ConsoleCommand;
use crate::i18n;

impl App {
    pub(super) fn run_console(&mut self, command: ConsoleCommand) {
        match command {
            ConsoleCommand::Share(text) => {
                if let Some(link) = &self.output_share {
                    link.share(self.console.share_mut(), text);
                }
            }
            ConsoleCommand::HideShare => self.console.share_mut().clear(),
            ConsoleCommand::ClearOutput => {
                self.console.clear_output();
            }
            ConsoleCommand::UpdateAndRetry => self.update_and_retry(),
        }
    }

    /// The update syncs the databases with the mirror, with the upgrader
    /// the app uses everywhere else, then the same install runs again.
    fn update_and_retry(&mut self) {
        let Some(packages) = self.console.mirror_retry().cloned() else {
            return;
        };

        let upgrader = upgrader(&self.current_snapshot(), self.garuda_update());
        self.run_at_once(retry_after_upgrade(packages, upgrader));
    }

    pub(super) fn take_output_share(&mut self, ctx: &egui::Context) {
        let Some(link) = &self.output_share else {
            return;
        };

        let done = link.take(self.console.share_mut());
        let paste = self.console.share().paste();
        if let Some(paste) =
            paste.filter(|_| done && self.prefs.setup.settings.diagnostics.copy_to_clipboard)
        {
            ctx.copy_text(paste.link.clone());
            self.toasts
                .add(ToastKind::Info, i18n::text(ctx, "share-link-copied"));
        }
    }
}
