use super::App;
use crate::command::Reveal;

impl App {
    /// Starts the reads only their pages start otherwise.
    pub(super) fn open_palette(&mut self) {
        self.palette.open();
        self.services.packages.open();
        if self.prefs.setup.settings.packages.show_aur {
            self.services.packages.open_aur();
        }

        self.services.units.open();
    }

    pub(super) fn run_reveal(&mut self, ctx: &egui::Context, reveal: &Reveal) {
        if let Reveal::Package { name, aur: _ } = reveal {
            self.services.packages.show(name);
        }

        let page = self.pages.reveal(reveal);
        self.navigate(ctx, page);
    }
}
