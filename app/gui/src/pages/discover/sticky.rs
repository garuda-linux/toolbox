//! The heading of the Discover group under the top edge of the list. It
//! stays there while the group scrolls and gives way to the next heading.

use egui::{CornerRadius, Rect, Ui, UiBuilder, pos2, vec2};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{page_viewport, section_heading};

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
    // The painting clip reaches up under the header, so the top edge is the
    // viewport's.
    let clip = ui.clip_rect();
    let top = page_viewport(ui.ctx()).map_or_else(|| clip.top(), |view| view.top());
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

    let at = pos2(clip.left(), top + next.min(0.0));
    let rect = Rect::from_min_size(at, vec2(clip.width(), height));
    let surface = tokens::current(ui.ctx()).window;
    ui.painter().rect_filled(rect, CornerRadius::ZERO, surface);

    let Some(group) = groups.get(index) else {
        return;
    };

    ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        section_heading(ui, group.icon, &group.title, &group.count);
    });
}
