//! The guide's pages at the left, under the parts of the summary, the
//! open one marked.

use std::sync::Arc;

use egui::accesskit::Role;
use egui::text::TextWrapping;
use egui::{
    Align, Align2, CornerRadius, FontSelection, Rect, Response, Sense, Ui, WidgetInfo, WidgetText,
    pos2, vec2,
};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::theme::RADIUS;
use toolbox_gui_kit::widgets::{focus_ring, rich};
use toolbox_guide::Guide;

use super::Jump;

const ENTRY_HEIGHT: f32 = 30.0;
const INSET: f32 = 10.0;
const MAX_LINES: usize = 2;

/// `reveal` scrolls the open page's entry into view, as it does when the
/// page changed.
pub(super) fn show(ui: &mut Ui, guide: &Guide, open: &str, reveal: bool) -> Option<Jump> {
    let tokens = ui.tokens();
    let mut jump = None;
    let mut tops = Vec::new();
    for part in guide.parts() {
        if let Some(title) = &part.title {
            ui.add_space(tokens.sizes.card_gap);
            let caps = title.to_uppercase();
            let heading = ui.label(rich(ui, &caps, Weight::SemiBold, SMALL, tokens.secondary));
            tops.push(heading.rect.top());
        }

        let pages = guide.pages().get(part.pages.clone()).unwrap_or_default();
        for page in pages {
            let selected = page.file == open;
            let shown = entry(ui, &page.title, selected);
            tops.push(shown.rect.top());
            if selected && reveal {
                reveal_whole(ui, &tops, shown.rect.center().y);
            }

            if shown.clicked() {
                jump = Some(Jump::Page {
                    file: page.file.clone(),
                    anchor: None,
                });
            }
        }
    }

    jump
}

/// Scrolls the list so `middle` sits about in the middle of its view, the
/// view starting on the last row in `tops` above the centred spot.
/// Centring alone lands anywhere and cuts the top row through its text.
fn reveal_whole(ui: &Ui, tops: &[f32], middle: f32) {
    let wanted = middle - ui.clip_rect().height() / 2.0;
    let Some(&top) = tops
        .iter()
        .rev()
        .find(|top| **top <= wanted)
        .or_else(|| tops.first())
    else {
        return;
    };

    let x = ui.min_rect().x_range();
    let row = Rect::from_x_y_ranges(x, top..=middle);
    ui.scroll_to_rect(row, Some(Align::Min));
}

/// One page of the list, a link that shows whether it is open. A title
/// wider than the entry wraps to a second line, and one longer still ends
/// in "…" and shows whole in a tooltip.
pub(super) fn entry(ui: &mut Ui, title: &str, selected: bool) -> Response {
    let tokens = ui.tokens();
    let width = ui.available_width();
    let (weight, colour) = if selected {
        (Weight::SemiBold, tokens.on_tint)
    } else {
        (Weight::Regular, tokens.text)
    };

    let text = WidgetText::from(rich(ui, title, weight, BODY, colour));
    let mut job = Arc::unwrap_or_clone(text.into_layout_job(
        ui.style(),
        FontSelection::Default,
        Align::Center,
    ));

    job.wrap = TextWrapping {
        max_width: INSET.mul_add(-2.0, width).max(0.0),
        max_rows: MAX_LINES,
        break_anywhere: false,
        overflow_character: Some('…'),
    };

    let galley = ui.painter().layout_job(job);
    let cut = galley.elided;

    // One line keeps the entry's height. A second adds its own.
    let line = galley.rows.first().map_or(0.0, |row| row.height());
    let height = ENTRY_HEIGHT + (galley.size().y - line).max(0.0);
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Link, ui.is_enabled(), title));
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_selected(selected));

    if ui.is_rect_visible(rect) {
        let fill = if selected {
            Some(tokens.accent_tint)
        } else if response.hovered() {
            Some(tokens.hover)
        } else {
            None
        };

        if let Some(fill) = fill {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(RADIUS), fill);
        }

        let room = rect.shrink2(vec2(INSET, 0.0));
        let at = Align2::LEFT_CENTER
            .align_size_within_rect(galley.size(), room)
            .min;

        ui.painter().galley(pos2(at.x, at.y), galley, colour);
    }

    focus_ring(ui, &response, RADIUS);
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    if cut {
        response.on_hover_text(title)
    } else {
        response
    }
}
