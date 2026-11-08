//! Every timing of the app's motion in one place, and the few ways it
//! moves: things that appear and leave, and jumps for what the keyboard
//! started.
//!
//! A productivity tool moves little. Motion is opacity, small offsets
//! and timing, never scale. Entering eases out, leaving runs straight,
//! except a sheet, which speeds up as it leaves ([`slide`]).
//! With less motion asked for ([`theme::reduced_motion`]) every duration
//! is zero, so nothing moves and nothing asks for extra frames.

#[cfg(test)]
mod tests;

use egui::emath::easing;
use egui::{Context, Id};

use crate::theme;

/// The Console sheet sliding up.
pub const SHEET_ENTER: f32 = 0.18;
/// The Console sheet sliding away.
pub const SHEET_EXIT: f32 = 0.14;
/// A note fading in and dropping into place.
pub const TOAST_ENTER: f32 = 0.16;
/// A note fading out.
pub const TOAST_EXIT: f32 = 0.15;
/// How far a note drops while it appears, in points.
pub const TOAST_DROP: f32 = 8.0;
/// The notes below one that left moving up.
pub const RESTACK: f32 = 0.15;
/// A page's content taking the place of its skeleton.
pub const CONTENT_FADE: f32 = theme::ANIMATION_TIME;
/// A pill button's hover fill.
pub const HOVER_FADE: f32 = 0.08;
/// The Console sheet both ways.
pub const SHEET: Timing = Timing::new(SHEET_ENTER, SHEET_EXIT);
/// The Changes popover rising from the pill.
pub const POPOVER_ENTER: f32 = 0.14;
/// How far the popover rises while it appears, in points.
pub const POPOVER_RISE: f32 = 6.0;
/// Rows dimming for a running plan and back.
pub const DIM_FADE: f32 = 0.15;
/// How long a key's wish to skip a switch's slide holds, so the change it
/// queued arrives within it.
const KEYBOARD_HOLD: f64 = 1.0;

/// Another page or tab fading in and rising into place.
pub const PAGE_ENTER: f32 = 0.15;
/// How far another page or tab rises while it appears, in points.
pub const PAGE_RISE: f32 = 8.0;

/// The chosen segment's pill moving to another segment.
pub const SEGMENT_SLIDE: f32 = 0.15;

/// How long it takes to show and to hide something.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timing {
    pub enter: f32,
    pub exit: f32,
}

impl Timing {
    pub const fn new(enter: f32, exit: f32) -> Self {
        Self { enter, exit }
    }

    pub const fn both(seconds: f32) -> Self {
        Self::new(seconds, seconds)
    }
}

/// `seconds`, or none when less motion is asked for.
pub fn duration(ctx: &Context, seconds: f32) -> f32 {
    if theme::reduced_motion(ctx) {
        0.0
    } else {
        seconds
    }
}

/// How far `id` is shown, from 0 to 1, moving toward `shown`. The first
/// call for an id takes the end at once, so what was there before the
/// window opened does not appear again.
pub fn presence(ctx: &Context, id: Id, shown: bool, timing: Timing) -> f32 {
    let seconds = if shown { timing.enter } else { timing.exit };
    let linear = ctx.animate_bool_with_time(id, shown, duration(ctx, seconds));
    if shown {
        easing::cubic_out(linear)
    } else {
        linear
    }
}

/// How far a sheet that slides from an edge is out, from 0 to 1.
///
/// It moves toward `shown` and leaves by speeding up, so it falls away
/// from the window rather than stopping short of the edge. The first call
/// for an id takes the end at once.
pub fn slide(ctx: &Context, id: Id, shown: bool, timing: Timing) -> f32 {
    let seconds = if shown { timing.enter } else { timing.exit };
    let linear = ctx.animate_bool_with_time(id, shown, duration(ctx, seconds));
    if shown {
        easing::cubic_out(linear)
    } else {
        1.0 - easing::cubic_in(1.0 - linear)
    }
}

/// [`presence`] for something new, such as a note just raised: the first
/// call for an id starts it hidden, so it appears.
pub fn fresh_presence(ctx: &Context, id: Id, shown: bool, timing: Timing) -> f32 {
    let seen = ctx.data_mut(|data| {
        std::mem::replace(data.get_temp_mut_or_default::<bool>(seen_id(id)), true)
    });

    if !seen {
        settle(ctx, id, false);
    }

    presence(ctx, id, shown, timing)
}

/// Puts the animation `id` at its end for `value` at once, as for a
/// change the keyboard started.
pub fn settle(ctx: &Context, id: Id, value: bool) {
    ctx.animate_bool_with_time(id, value, 0.0);
}

/// Marks that a key changed what lies inside the ui of `id`, such as a
/// row whose switch Space flipped. [`keyboard_marked`] finds it.
pub fn mark_keyboard(ctx: &Context, id: Id) {
    let now = ctx.input(|input| input.time);
    ctx.data_mut(|data| data.insert_temp(keyboard_id(id), now));
}

/// A key changed what lies inside `ui` a moment ago, so it jumps.
pub fn keyboard_marked(ui: &egui::Ui) -> bool {
    let now = ui.input(|input| input.time);
    ui.data(|data| {
        ui.stack().iter().any(|node| {
            data.get_temp::<f64>(keyboard_id(node.scope_id))
                .is_some_and(|at| now - at <= KEYBOARD_HOLD)
        })
    })
}

fn seen_id(id: Id) -> Id {
    id.with("toolbox_gui_kit::motion::seen")
}

fn keyboard_id(id: Id) -> Id {
    id.with("toolbox_gui_kit::motion::keyboard")
}
