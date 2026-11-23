//! The sidebar's head: the search entry above the page list, with the
//! collapse toggle beside it, as System Settings has no title there.

use egui::{Id, Ui, vec2};
use toolbox_gui_kit::icons;
use toolbox_gui_kit::widgets::{
    SIDE_INSET, SearchTexts, icon_button_width, nav_row, push_height, search_field_unrequested,
    sidebar_button,
};

use super::density::Density;
use crate::command::{CommandSender, UiCommand};
use crate::{i18n, page_ui};

/// Kirigami's smallSpacing, between the search field and the toggle beside it.
const SEARCH_GAP: f32 = 4.0;

/// The collapsed rail's own row for the expand toggle.
pub(super) fn rail_toggle(ui: &mut Ui, commands: &CommandSender) {
    let text = i18n::text(ui.ctx(), "sidebar-expand");
    toggle(ui, icons::SIDEBAR_EXPAND_LEFT, &text, commands);
}

/// The search of System Settings: a search field at the top of the sidebar.
/// Typing opens the app's search, the palette. The collapsed rail keeps a
/// button for it.
pub(super) fn search(ui: &mut Ui, density: Density, expanded: bool, commands: &CommandSender) {
    ui.add_space(density.top_gap);
    let name = i18n::text(ui.ctx(), "sidebar-search");
    if !expanded {
        if nav_row(ui, page_ui::SEARCH_ICON, &name, false, density.look()).clicked() {
            commands.send(UiCommand::OpenPalette);
        }

        return;
    }

    let id = ui.make_persistent_id("toolbox_gui::sidebar::search");
    let mut query = ui
        .data(|data| data.get_temp::<String>(id))
        .unwrap_or_default();

    let hint = i18n::text(ui.ctx(), "sidebar-search-hint");
    let clear = i18n::text(ui.ctx(), "search-clear");
    let collapse = i18n::text(ui.ctx(), "sidebar-collapse");
    let texts = SearchTexts {
        name: &name,
        hint: &hint,
        clear: &clear,
    };

    let right = entries_right(ui).unwrap_or_else(|| ui.max_rect().right());

    // A row of the field's height, so the toggle centres on the field.
    let height = push_height(ui);
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), height),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = SEARCH_GAP;
            ui.add_space(SIDE_INSET);

            // Tab follows the row from left to right, so the field is added
            // first and leaves the toggle its slot at the right edge.
            let end = (right - SIDE_INSET).min(ui.max_rect().right());
            let width = (end - icon_button_width() - SEARCH_GAP - ui.cursor().left()).max(0.0);
            let response = search_field_unrequested(ui, "sidebar", &mut query, texts, width);
            if response.changed() && !query.is_empty() {
                crate::shell::palette::pass_query(ui.ctx(), std::mem::take(&mut query));
                commands.send(UiCommand::OpenPalette);
            }

            toggle(ui, icons::SIDEBAR_COLLAPSE_LEFT, &collapse, commands);
        },
    );

    ui.data_mut(|data| data.insert_temp(id, query));
}

fn entries_right_id() -> Id {
    Id::unique("toolbox_gui::sidebar::entries_right")
}

/// The right edge the page list's entries reached in the last frame. The
/// list's scrollbar moves it in when it takes room.
fn entries_right(ui: &Ui) -> Option<f32> {
    ui.data(|data| data.get_temp(entries_right_id()))
}

/// Keeps where the entries end for the next frame's head, and asks for that
/// frame when it moved.
pub(super) fn keep_entries_right(ui: &Ui, right: f32) {
    if entries_right(ui).is_some_and(|kept| (kept - right).abs() < 0.01) {
        return;
    }

    ui.data_mut(|data| data.insert_temp(entries_right_id(), right));
    ui.ctx().request_repaint();
}

fn toggle(ui: &mut Ui, icon: icons::Icon, text: &str, commands: &CommandSender) {
    if sidebar_button(ui, icon, text).clicked() {
        commands.send(UiCommand::ToggleSidebar);
    }
}
