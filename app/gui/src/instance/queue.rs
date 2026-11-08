//! Commands ask about the window's queue on the bus's threads. The
//! questions wait here until the app's logic, which owns the queue,
//! answers them. The logic runs while the window is hidden too.

#[cfg(test)]
mod tests;

use toolbox_instance::bus::queue::{Ask, Desk};

use super::Arrivals;

impl Desk for Arrivals {
    fn ask(&self, ask: Ask) {
        if self.asks.0.send(ask).is_err() {
            tracing::warn!("the queue's questions have nowhere to go");
        }

        if let Some(ctx) = self.ctx.get() {
            ctx.request_repaint();
        }
    }
}

impl Arrivals {
    #[must_use = "an unanswered question waits until its caller gives up"]
    pub fn questions(&self) -> Vec<Ask> {
        self.asks.1.try_iter().collect()
    }
}
