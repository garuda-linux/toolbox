//! Rows that wait for a running plan, drawn faded (spotifast's collection
//! rows). The page's banner says why, the fade shows which rows it means.
//! And the picture of a row the user can't change, faded as its dim title.

use egui::{Rect, Ui, UiBuilder};

use crate::motion::{self, Timing};
use crate::tokens::{self, Tokens};

const BUSY_OPACITY: f32 = 0.55;

/// `body` at reduced opacity while `busy`, fading there and back. A scope
/// wraps it either way, so a plan that starts or ends keeps the ids
/// inside.
pub fn dimmed<R>(ui: &mut Ui, busy: bool, body: impl FnOnce(&mut Ui) -> R) -> R {
    // The next automatic id is stable while the widgets before it are, as
    // in a row with its own scope id, and differs between two calls in one
    // ui.
    let id = ui.next_auto_id().with("toolbox_gui_kit::dim");
    let dim = motion::presence(ui.ctx(), id, busy, Timing::both(motion::DIM_FADE));
    ui.scope(|ui| {
        if dim > 0.0 {
            ui.multiply_opacity(egui::lerp(1.0..=BUSY_OPACITY, dim));
        }

        body(ui)
    })
    .inner
}

/// How far `tokens.dim` sits from the card towards the text, from 0 to 1.
/// A picture painted at this opacity on the card fades as far as a title
/// in `tokens.dim` does.
pub fn dim_opacity(tokens: &Tokens) -> f32 {
    let distance = |colour: egui::Color32| {
        colour
            .to_array()
            .iter()
            .zip(tokens.card.to_array())
            .take(3)
            .map(|(one, other)| f32::from(one.abs_diff(other)))
            .sum::<f32>()
    };

    let full = distance(tokens.text);
    if full <= 0.0 {
        return 1.0;
    }

    (distance(tokens.dim) / full).clamp(0.0, 1.0)
}

/// `body` painting a picture into `at`, at [`dim_opacity`] when `dim`, so
/// the picture of a row whose title is dim reads with it as one. The Ui
/// it gets lays nothing out in `ui`.
pub fn dim_picture<R>(ui: &mut Ui, at: Rect, dim: bool, body: impl FnOnce(&Ui) -> R) -> R {
    let mut picture = ui.new_child(UiBuilder::new().max_rect(at));
    if dim {
        picture.multiply_opacity(dim_opacity(&tokens::current(ui.ctx())));
    }

    body(&picture)
}
