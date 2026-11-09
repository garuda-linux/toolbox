//! Commands ask about the window's queue on the bus's threads. The
//! questions wait here until the app's logic, which owns the queue,
//! answers them. The logic runs while the window is hidden too.

#[cfg(test)]
mod tests;

use crossbeam_channel::TrySendError;
use toolbox_instance::bus::queue::{Ask, Desk, Refusal};

use super::Arrivals;

impl Desk for Arrivals {
    fn ask(&self, ask: Ask) {
        match self.asks.0.try_send(ask) {
            Ok(()) => {}
            Err(TrySendError::Full(ask)) => {
                tracing::warn!("refused a queue command: too many wait for an answer");

                // An error only means the caller gave up already.
                ask.answer.send(Err(Refusal::TooMany)).ok();
                return;
            }
            Err(TrySendError::Disconnected(_)) => {
                tracing::warn!("the queue's questions have nowhere to go");
            }
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
