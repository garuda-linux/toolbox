//! Shares a text as an encrypted paste on the Garuda paste server, one
//! upload at a time per [`Sharer`]. The Diagnostics report and the
//! Console's output both go out through it.
//!
//! The HTTPS request blocks, so it runs on a blocking thread and answers
//! through a [`ShareSink`].

mod problem;
#[cfg(test)]
mod tests;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub use problem::UploadProblem;
use tokio::runtime::Handle;
use toolbox_privatebin::{Paste, Transport, upload};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShareNote {
    Started,
    Uploaded(Result<Paste, UploadProblem>),
}

pub trait ShareSink: Send + Sync {
    fn note(&self, note: ShareNote);
}

/// The last upload of one text, as its view shows it. Each stage holds
/// only its own data, so a running upload has no old link and a done one
/// no reason.
#[derive(Clone, Default, PartialEq, Eq)]
pub enum Upload {
    #[default]
    Idle,
    Running,
    /// The paste's link with the key and its delete link. Never logged.
    Done(Paste),
    Failed(UploadProblem),
}

/// Uploads texts. Clones share one guard, so a second click cannot post
/// while the first upload runs.
#[derive(Clone)]
pub struct Sharer {
    runtime: Handle,
    transport: Arc<dyn Transport>,
    sink: Arc<dyn ShareSink>,
    uploading: Arc<AtomicBool>,
}

impl std::fmt::Debug for Upload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => f.write_str("Idle"),
            Self::Running => f.write_str("Running"),
            Self::Done(_) => f.debug_tuple("Done").finish_non_exhaustive(),
            Self::Failed(problem) => f.debug_tuple("Failed").field(problem).finish(),
        }
    }
}

impl std::fmt::Debug for Sharer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sharer")
            .field("uploading", &self.uploading.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

impl Upload {
    /// Takes a note. Returns whether a link arrived.
    pub fn apply(&mut self, note: ShareNote) -> bool {
        let done = matches!(note, ShareNote::Uploaded(Ok(_)));
        *self = match note {
            ShareNote::Started => Self::Running,
            ShareNote::Uploaded(Ok(paste)) => Self::Done(paste),
            ShareNote::Uploaded(Err(problem)) => Self::Failed(problem),
        };
        done
    }

    pub fn start(&mut self) {
        *self = Self::Running;
    }

    pub fn clear(&mut self) {
        *self = Self::Idle;
    }

    pub const fn running(&self) -> bool {
        matches!(self, Self::Running)
    }

    pub const fn paste(&self) -> Option<&Paste> {
        match self {
            Self::Done(paste) => Some(paste),
            Self::Idle | Self::Running | Self::Failed(_) => None,
        }
    }

    pub const fn problem(&self) -> Option<&UploadProblem> {
        match self {
            Self::Failed(problem) => Some(problem),
            Self::Idle | Self::Running | Self::Done(_) => None,
        }
    }
}

impl Sharer {
    #[must_use = "the sharer does nothing until asked to share"]
    pub fn new(runtime: Handle, transport: Arc<dyn Transport>, sink: Arc<dyn ShareSink>) -> Self {
        Self {
            runtime,
            transport,
            sink,
            uploading: Arc::default(),
        }
    }

    /// Uploads `text` on a blocking thread. Returns whether it started:
    /// not while an upload of this sharer runs.
    pub fn share(&self, text: String) -> bool {
        if self.uploading.swap(true, Ordering::AcqRel) {
            tracing::debug!("an upload runs already, so the next one is refused");
            return false;
        }

        let transport = Arc::clone(&self.transport);
        let sink = Arc::clone(&self.sink);
        let uploading = Arc::clone(&self.uploading);

        tracing::info!("uploading a text of {} bytes", text.len());
        self.sink.note(ShareNote::Started);

        let work = self
            .runtime
            .spawn_blocking(move || upload(transport.as_ref(), &text).map_err(UploadProblem::from));

        // A panic on the blocking thread ends as a failed upload.
        self.runtime.spawn(async move {
            let result = work.await.unwrap_or_else(|error| {
                tracing::error!("the upload stopped: {error}");
                Err(UploadProblem::stopped())
            });

            // Never the link: its fragment is the key to the text.
            match &result {
                Ok(_) => tracing::info!("the text was uploaded"),
                Err(problem) => tracing::error!(
                    "the text was not uploaded ({}): {}",
                    problem.kind.as_str(),
                    problem.detail
                ),
            }

            // Before the note, so the view may share again once it hears it.
            uploading.store(false, Ordering::Release);
            sink.note(ShareNote::Uploaded(result));
        });

        true
    }
}
