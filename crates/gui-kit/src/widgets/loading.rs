//! Three pulsing dots for a short wait inline, where a skeleton has no
//! shape to take (Rerun's loading indicator). Screen readers hear a
//! progress indicator named by what is waited for.

use std::f32::consts::TAU;
use std::time::Duration;

use egui::{Rect, Response, Role, Sense, TextStyle, Ui, WidgetInfo, pos2, vec2};

use super::line;
use super::mark_word::mark_and_word;
use crate::fonts::{BODY, Weight};
use crate::{motion, theme, tokens};

/// About thirty frames a second are enough for dots, the display's rate
/// would only cost power.
const FRAME: Duration = Duration::from_millis(33);
const DOTS: u8 = 3;
const SPEED: f32 = 1.2;

pub fn loading_indicator(ui: &mut Ui, reason: &str) -> Response {
    let response = loading_dots(ui);
    response.widget_info(|| WidgetInfo::labeled(Role::ProgressIndicator, true, reason));
    response
}

/// The dots beside words that already say what is waited for, such as a
/// status tag. They carry no name, so a screen reader hears the words once.
pub(super) fn loading_dots(ui: &mut Ui) -> Response {
    let height = ui.text_style_height(&TextStyle::Body);
    let (rect, response) = ui.allocate_exact_size(dots_size(height), Sense::hover());
    paint_dots(ui, rect);
    response
}

/// The dots and `text` beside them, one node for screen readers.
pub fn loading_label(ui: &mut Ui, text: &str) -> Response {
    let tokens = tokens::current(ui.ctx());
    let height = ui.text_style_height(&TextStyle::Body);
    let galley = line(ui, text, Weight::Regular, BODY, tokens.secondary);
    mark_and_word(
        ui,
        (dots_size(height), tokens.sizes.text_to_icon),
        galley,
        (Role::ProgressIndicator, text),
        paint_dots,
    )
}

fn dots_size(height: f32) -> egui::Vec2 {
    vec2(height * 1.6, height)
}

/// With less motion asked for, or in a minimized or covered window, the
/// dots stand still and ask for no frames.
fn paint_dots(ui: &Ui, rect: Rect) {
    if !ui.is_rect_visible(rect) {
        return;
    }

    let still = theme::reduced_motion(ui.ctx()) || !ui.input(motion::window_in_sight);
    if !still {
        ui.ctx().request_repaint_after(FRAME);
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::as_conversions,
        reason = "the phase only needs a float's precision"
    )]
    let time = ui.input(|input| input.time) as f32;
    let accent = tokens::current(ui.ctx()).accent;
    let radius = rect.height() / 7.0;
    let step = rect.width() / f32::from(DOTS);
    for dot in 0..DOTS {
        let phase = f32::from(dot).mul_add(-TAU / f32::from(DOTS), time * SPEED * TAU);
        let pulse = if still {
            1.0
        } else {
            0.5_f32.mul_add(phase.sin(), 0.5)
        };

        let center = pos2(
            (f32::from(dot) + 0.5).mul_add(step, rect.left()),
            rect.center().y,
        );

        let color = accent.gamma_multiply(0.65_f32.mul_add(pulse, 0.35));

        ui.painter().circle_filled(center, radius, color);
    }
}
