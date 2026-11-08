//! What the app does with a click on Diagnostics and with the service's
//! notes. With the setting on, a finished report and an upload's link go
//! to the clipboard at once. The delete link never does.

use toolbox_gui_kit::toasts::ToastKind;
use toolbox_system::diagnostics::page::Changed;

use super::App;
use crate::command::DiagnosticsCommand;
use crate::i18n;

impl App {
    pub(super) fn run_diagnostics(&mut self, command: DiagnosticsCommand) {
        match command {
            DiagnosticsCommand::Collect(selection) => self.diagnostics.collect(selection),
            DiagnosticsCommand::Cancel => self.diagnostics.cancel(),
            DiagnosticsCommand::Upload => {
                self.diagnostics.upload();
            }
            DiagnosticsCommand::HideShare => self.diagnostics.clear_upload(),
        }
    }

    pub(super) fn take_diagnostics(&mut self, ctx: &egui::Context) {
        let changes = self.diagnostics.take();
        if !self.prefs.setup.settings.diagnostics.copy_to_clipboard {
            return;
        }

        for changed in changes {
            let (text, id) = match changed {
                Changed::Report { ready: true } => (
                    self.diagnostics.page().report.clone(),
                    "diagnostics-report-copied",
                ),
                Changed::Upload { done: true } => {
                    let Some(paste) = self.diagnostics.page().share.paste() else {
                        continue;
                    };

                    (paste.link.clone(), "share-link-copied")
                }
                Changed::Status
                | Changed::Report { ready: false }
                | Changed::Upload { done: false } => {
                    continue;
                }
            };

            ctx.copy_text(text);
            self.toasts.add(ToastKind::Info, i18n::text(ctx, id));
        }
    }
}
