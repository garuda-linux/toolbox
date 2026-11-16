//! A read of the system in progress, on a line of its own above the app's
//! entries, so it never covers a page or the Console.

use egui::{Direction, Frame, Layout, Panel, Ui, vec2};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{NAV_INSET, loading_indicator};

use crate::i18n;

/// One entry tall, the dots where the entries' icons start. Collapsed,
/// the dots alone, still named for screen readers.
pub(super) fn show(ui: &mut Ui, row: f32, expanded: bool) {
    let text = i18n::text(ui.ctx(), "status-reading");
    Panel::bottom("sidebar-reading")
        .show_separator_line(false)
        .frame(Frame::NONE)
        .exact_size(row)
        .show(ui, |ui| {
            if expanded {
                ui.horizontal_centered(|ui| {
                    ui.add_space(NAV_INSET);
                    ui.loading(&text);
                });
            } else {
                let centred = Layout::centered_and_justified(Direction::LeftToRight);
                ui.allocate_ui_with_layout(vec2(row, row), centred, |ui| {
                    loading_indicator(ui, &text);
                });
            }
        });
}
