//! A file the Boot options page's chooser picked, which comes back as an
//! answer of its service. Every click of the page goes to the service.

use toolbox_core::features::boot::edit::Edit;

use super::App;

impl App {
    pub(super) fn edit_boot(&mut self, ctx: &egui::Context, change: Edit) {
        let snapshot = self.current_snapshot();
        let handled = self
            .services
            .boot
            .edit(change, self.queue.changes(), &snapshot);

        self.take_handled(ctx, handled);
    }
}
