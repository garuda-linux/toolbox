//! The sharer answers on its own threads, so its notes come through a
//! channel that wakes the frame.

mod preview;
#[cfg(test)]
mod tests;
pub mod view;

use std::sync::Arc;

use crossbeam_channel::Receiver;
use toolbox_core::features::diagnostics::Known;
use toolbox_privatebin::Transport;
use toolbox_system::share::{ShareNote, ShareSink, Sharer, Upload};

use crate::waking::{self, Wake, Waking};

#[derive(Debug)]
pub struct ShareLink {
    sharer: Sharer,
    notes: Receiver<ShareNote>,
    /// What the shared text must not name.
    known: Known,
}

struct Sink(Waking<ShareNote>);

impl ShareSink for Sink {
    fn note(&self, note: ShareNote) {
        self.0.send(note);
    }
}

impl ShareLink {
    /// What `link` keeps out of the shared text, nothing without a link.
    pub fn known_of(link: Option<&Self>) -> Known {
        link.map(|link| link.known.clone()).unwrap_or_default()
    }

    /// Uploads `text` into `upload`. Returns whether it started: not
    /// without text or while an upload of this link runs.
    pub fn share(&self, upload: &mut Upload, text: String) -> bool {
        if text.is_empty() || upload.running() || !self.sharer.share(text) {
            return false;
        }

        upload.start();
        true
    }

    /// Takes the notes that came since the last frame into `upload`.
    /// Returns whether a link arrived.
    pub fn take(&self, upload: &mut Upload) -> bool {
        // Every note counts, so no early stop at the first link.
        let mut done = false;
        for note in self.notes.try_iter() {
            done |= upload.apply(note);
        }

        done
    }
}

/// A sharer on `runtime` that posts through `transport`. Each note calls
/// `wake`.
#[must_use = "the link is the only way to the sharer"]
pub fn spawn(
    runtime: &tokio::runtime::Handle,
    wake: Wake,
    transport: Arc<dyn Transport>,
    known: Known,
) -> ShareLink {
    let (sender, notes) = waking::channel(wake);
    let sink = Arc::new(Sink(sender));

    ShareLink {
        sharer: Sharer::new(runtime.clone(), transport, sink),
        notes,
        known,
    }
}
