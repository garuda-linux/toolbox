//! The empty state of a list that the search emptied. Every page says the
//! same title and offers the same Clear search button, so only the detail
//! names what the page searches.

use egui::Ui;
use toolbox_gui_kit::icons;
use toolbox_gui_kit::widgets::empty_state_action;

use crate::i18n;

/// Returns whether Clear search was pressed, the caller empties its search.
pub fn search_empty_state(ui: &mut Ui, search: &str, detail: &str) -> bool {
    let ctx = ui.ctx().clone();
    let title = i18n::text_with(&ctx, "search-nothing-matches", &[("search", search)]);
    let clear = i18n::text(&ctx, "search-clear");
    empty_state_action(ui, icons::SYSTEM_SEARCH, &title, detail, &clear).clicked()
}
