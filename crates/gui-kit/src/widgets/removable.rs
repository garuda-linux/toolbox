//! A tag with its own remove button inside, for a list someone edits
//! piece by piece, such as kernel parameters.

use egui::{Role, Sense, Ui, WidgetInfo, pos2, vec2};

use super::chip::TAG_SCALE;
use super::{Tone, focus_ring, line};
use crate::fonts::Weight;
use crate::ui_ext::icon_image;
use crate::{icons, tokens};

/// The remove button's square, the smallest target a pointer hits well.
const TARGET: f32 = 24.0;
/// Room around the target inside the tag.
const INSET: f32 = 2.0;
const PADDING_X: f32 = 10.0;
pub const GLYPH: f32 = 12.0;

/// `text` on a tag with a remove cross that `alt` names. Returns whether
/// the cross was clicked.
pub fn removable_tag(ui: &mut Ui, text: &str, alt: &str) -> bool {
    let tokens = tokens::current(ui.ctx());
    let (fill, ink) = Tone::Neutral.colors(&tokens);
    let galley = line(ui, text, Weight::SemiBold, TAG_SCALE, ink);
    let height = INSET.mul_add(2.0, TARGET);
    let width = PADDING_X + galley.size().x + INSET + TARGET + INSET;
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());

    let words = rect.with_max_x(rect.left() + PADDING_X + galley.size().x);
    let label = ui.interact(
        words,
        ui.next_auto_id().with(("removable", text)),
        Sense::hover(),
    );

    label.widget_info(|| WidgetInfo::labeled(Role::Label, true, text));

    let cross_rect = egui::Rect::from_min_size(
        pos2(rect.right() - INSET - TARGET, rect.top() + INSET),
        vec2(TARGET, TARGET),
    );

    let cross = ui.interact(cross_rect, label.id.with("remove"), Sense::click());
    cross.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), alt));
    let cross = cross.on_hover_text(alt);

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect_filled(rect, height / 2.0, fill);
        if cross.hovered() {
            painter.circle_filled(cross_rect.center(), TARGET / 2.0, tokens.pressed);
        }

        let at = pos2(
            rect.left() + PADDING_X,
            rect.center().y - galley.size().y / 2.0,
        );

        painter.galley(at, galley, ink);
        icon_image(ui, icons::WINDOW_CLOSE, tokens.secondary)
            .fit_to_exact_size(vec2(GLYPH, GLYPH))
            .paint_at(
                ui,
                egui::Rect::from_center_size(cross_rect.center(), vec2(GLYPH, GLYPH)),
            );
    }

    focus_ring(ui, &cross, u8::MAX);
    cross.clicked()
}
