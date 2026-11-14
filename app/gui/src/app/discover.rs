use super::App;
use crate::command::DiscoverCommand;
use crate::page::PageId;

impl App {
    pub(super) fn run_discover(&mut self, command: DiscoverCommand) {
        match command {
            DiscoverCommand::Reveal(title) => {
                self.pages.reveal_app(&title);
                self.state.page = PageId::Discover;
            }
            // `TogglePages` takes these, see `hand_to_services`.
            DiscoverCommand::Toggle { app: _, wish: _ } => {
                tracing::warn!("no page service took {command:?}");
            }
        }
    }
}
