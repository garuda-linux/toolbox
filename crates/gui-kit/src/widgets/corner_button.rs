//! A small icon button over the top right corner of an area, such as a
//! code block's copy button. It shows only while the pointer is on the
//! area or the button holds the focus. Hidden, it keeps its Tab stop and
//! its node for screen readers.

use egui::{CornerRadius, Rect, Response, Role, Sense, Ui, UiBuilder, Vec2, WidgetInfo, pos2};

use super::buttons::{SMALL_BUTTON, paint_small_button};
use crate::icons::Icon;
use crate::theme::CONTROL_RADIUS;
use crate::{motion, tokens};

pub fn corner_button(
    ui: &mut Ui,
    area: Rect,
    line_middle: f32,
    icon: Icon,
    name: &str,
) -> Response {
    let id = ui.scope_id().with(("corner_button", name));
    let inset = (line_middle - area.top() - SMALL_BUTTON / 2.0).max(0.0);
    let rect = Rect::from_min_size(
        pos2(area.right() - inset - SMALL_BUTTON, area.top() + inset),
        Vec2::splat(SMALL_BUTTON),
    );

    let shown = ui.rect_contains_pointer(area) || ui.memory(|memory| memory.has_focus(id));

    // Hidden, it takes no click, so a press there reaches the text under it.
    let sense = if shown {
        Sense::click()
    } else {
        Sense::focusable_noninteractive()
    };

    let response = ui.interact(rect, id, sense);
    response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), name));

    let opacity = motion::presence(ui.ctx(), id.with("shown"), shown, motion::CORNER_BUTTON);
    if opacity > 0.0 {
        let tokens = tokens::current(ui.ctx());
        let mut over = ui.new_child(UiBuilder::new().max_rect(rect));
        over.multiply_opacity(opacity);

        // The backing keeps a long line scrolled under it from showing through.
        over.painter()
            .rect_filled(rect, CornerRadius::same(CONTROL_RADIUS), tokens.field);
        paint_small_button(&over, rect, &response, icon, tokens.hover);
    }

    if shown {
        response.on_hover_text(name)
    } else {
        response
    }
}
