//! The heading of the Discover group under the top edge of the list. It
//! stays there while the group scrolls and gives way to the next heading.

use egui::{CornerRadius, Rect, Ui, UiBuilder, pos2, vec2};
use toolbox_gui_kit::widgets::section_heading;
use toolbox_gui_kit::{frost, theme, tokens};

use super::app_row::AppGroup;

/// Where a group's heading and rows lie. The positions count from the top
/// of the list's content, so they hold while the list scrolls.
#[derive(Clone, Copy, Debug)]
pub struct Mark {
    pub top: f32,
    pub heading_bottom: f32,
    pub end: f32,
}

/// Paints over the list the heading of the group the top edge is in, once
/// its own heading has scrolled out. `start` is where the list's content
/// began this frame.
pub fn paint(ui: &mut Ui, groups: &[AppGroup], marks: &[Mark], start: f32) {
    // The list scrolls in its own column, whose clip starts at its top edge,
    // so the clip is the viewport the heading sticks to.
    let clip = ui.clip_rect();
    let top = clip.top();
    let seen = top - start;
    let Some((index, &mark)) = marks
        .iter()
        .enumerate()
        .rev()
        .find(|(_, mark)| mark.top < seen)
    else {
        return;
    };

    let height = mark.heading_bottom - mark.top;
    if mark.end <= seen + height {
        return;
    }

    // The next heading pushes this one up as it arrives.
    let next = marks
        .get(index + 1)
        .map_or(f32::INFINITY, |next| next.top - seen - height);

    // The body's own left padding, so the heading lines up with the rows.
    let padding = f32::from(tokens::current(ui.ctx()).sizes.page_padding.left);
    let at = pos2(clip.left(), top + next.min(0.0));
    let band = Rect::from_min_size(at, vec2(clip.width(), height));

    // The rows under the heading blur through the header's scrim, as the
    // page header does. Without the blur the band stays solid, so no row
    // shows through the heading.
    let tokens = tokens::current(ui.ctx());
    let cover = if frost::live(ui.ctx()) {
        frost::backdrop_within(ui.painter(), band, CornerRadius::ZERO);
        tokens.header_scrim
    } else if theme::translucent(ui.ctx()) {
        tokens.header_scrim
    } else {
        tokens.window
    };

    ui.painter().rect_filled(band, CornerRadius::ZERO, cover);

    let Some(group) = groups.get(index) else {
        return;
    };

    let rect = band.shrink2(vec2(padding, 0.0));
    ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        section_heading(ui, &group.title, &group.count);
    });
}
