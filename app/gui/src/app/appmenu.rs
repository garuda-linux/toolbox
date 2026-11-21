//! Keeps the global menu in step with the app.

use super::App;
use crate::appmenu::MenuState;
use crate::page::PageId;
use crate::{i18n, page_ui};

impl App {
    pub(super) fn follow_menu_bar(&mut self, ctx: &egui::Context) {
        let Some(shown) = &mut self.menu_bar else {
            return;
        };

        let state = MenuState {
            language: i18n::language(ctx),
            page: self.state.page,
            listed: PageId::ALL
                .into_iter()
                .filter(|page| page_ui::listed(ctx, *page))
                .collect(),
            sidebar_expanded: self.state.sidebar_expanded,
            console_open: self.state.console_open,
            queued: !self.queue.changes().is_empty(),
            zoom: self.state.zoom,
        };

        shown.follow(state, |id| i18n::text(ctx, id).into_owned());
    }
}
