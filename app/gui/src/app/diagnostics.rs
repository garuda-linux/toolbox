//! What the app does with a click on Diagnostics and with the service's
//! notes. With the setting on, a finished report and an upload's link go
//! to the clipboard at once. The delete link never does.

use super::App;
use crate::command::DiagnosticsCommand;

impl App {
    pub(super) fn run_diagnostics(&mut self, command: DiagnosticsCommand) {
        match command {
            DiagnosticsCommand::Collect(selection) => self.services.diagnostics.collect(selection),
            DiagnosticsCommand::Cancel => self.services.diagnostics.cancel(),
            DiagnosticsCommand::Upload => {
                self.services.diagnostics.upload();
            }
            DiagnosticsCommand::HideShare => self.services.diagnostics.clear_upload(),
        }
    }
}
