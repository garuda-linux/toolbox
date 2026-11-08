use super::App;
use crate::command::DiscoverCommand;
use crate::page::PageId;

impl App {
    pub(super) fn run_discover(&mut self, command: DiscoverCommand) {
        match command {
            DiscoverCommand::Toggle { app, wish } => {
                let apps = &self.catalogs.apps.apps;
                let snapshot = self.current_snapshot();
                let queued = self.queue.edit(&snapshot, |changes, snapshot| {
                    apps.toggle(&app, wish, snapshot, changes)
                });

                // A kept app's switch rests, so only a stale id or state
                // from an old frame lands here.
                if !queued {
                    tracing::warn!("the app {app} could not be queued");
                }
            }
            DiscoverCommand::Reveal(title) => {
                self.pages.reveal_app(&title);
                self.state.page = PageId::Discover;
            }
        }
    }
}
