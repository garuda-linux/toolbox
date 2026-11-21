//! The repaints a harness at rest still asks for. egui's own repaint
//! delay keeps only the soonest request, the observer sees every one.

use std::sync::Arc;
use std::time::Duration;

use egui::{RepaintCause, ViewportId};
use egui_kittest::Harness;
use parking_lot::Mutex;

#[derive(Debug, Default)]
pub struct Repaints {
    pub delays: Vec<Duration>,
    pub causes: Vec<RepaintCause>,
}

impl Repaints {
    #[must_use = "it only answers a question"]
    pub const fn is_empty(&self) -> bool {
        self.delays.is_empty()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Cover {
    Minimized,
    Occluded,
}

pub fn repaints_once_settled<State>(harness: &mut Harness<'_, State>) -> Repaints {
    harness.run();

    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = Arc::clone(&seen);

    // egui calls it with the context locked, so it may only store.
    harness.ctx.set_repaint_observer(move |info| {
        record.lock().push(info.delay);
    });

    harness.step();
    harness.ctx.set_repaint_observer(|_| {});

    // egui hands out a frame's causes only once the next one began.
    harness.step();

    let delays = std::mem::take(&mut *seen.lock());
    Repaints {
        delays,
        causes: harness.ctx.repaint_causes(),
    }
}

pub fn cover<State>(harness: &mut Harness<'_, State>, how: Cover) {
    let input = harness.input_mut();
    let Some(viewport) = input.viewports.get_mut(&ViewportId::ROOT) else {
        return;
    };

    match how {
        Cover::Minimized => viewport.minimized = Some(true),
        Cover::Occluded => viewport.occluded = Some(true),
    }
}
