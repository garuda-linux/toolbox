//! A notifier that records the bubbles instead of sending them.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::{Notification, Notifier};

#[derive(Debug)]
pub struct Recorder(Sender<Notification>);

impl Recorder {
    /// The notifier, and the receiver of every bubble it is asked to show.
    #[must_use = "the receiver is how a test reads the bubbles"]
    pub fn pair() -> (Arc<dyn Notifier>, Receiver<Notification>) {
        let (sender, receiver) = channel();
        (Arc::new(Self(sender)), receiver)
    }
}

impl Notifier for Recorder {
    fn show(&self, notification: Notification) {
        // A receiver that is gone means no test waits for the bubble.
        self.0.send(notification).ok();
    }
}
