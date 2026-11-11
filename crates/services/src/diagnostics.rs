//! What the Diagnostics page keeps between frames: the last report in its
//! plain and coloured forms and the upload, fed by the diagnostics
//! service of `toolbox-system`.
//!
//! The service answers on its own threads, so its notes come through a
//! channel that wakes the frame.

pub mod page;
pub mod service;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use crossbeam_channel::Receiver;
use toolbox_core::features::diagnostics::Selection;
use toolbox_privatebin::{RequestError, Transport};
use toolbox_pty::{Run, styled_lines};
use toolbox_system::DiagnosticSource;

use self::page::{Changed, Page, State};
use self::service::{DiagnosticsHandle, DiagnosticsNote, DiagnosticsSink, Request};
use crate::waking::{self, Wake, Waking};

/// The paste server's answer in a dry run: the upload goes nowhere and
/// the page still shows a link.
const DRY_RUN_ANSWER: &str = r#"{"status":0,"id":"dryrun0000000000","deletetoken":"dryrun"}"#;

pub struct DiagnosticsLink {
    handle: DiagnosticsHandle,
    notes: Receiver<DiagnosticsNote>,
}

/// Stands in for the paste server in a dry run and its self-test.
#[derive(Clone, Copy, Debug)]
pub struct DryRunTransport;

struct Sink(Waking<DiagnosticsNote>);

#[derive(Default)]
pub struct Diagnostics {
    page: Page,
    /// The report's lines with their colours, parsed once per report.
    lines: Vec<Vec<Run>>,
    /// `None` in tests that need no service: nothing is collected.
    link: Option<DiagnosticsLink>,
}

impl std::fmt::Debug for DiagnosticsLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiagnosticsLink").finish_non_exhaustive()
    }
}

impl std::fmt::Debug for Diagnostics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Diagnostics")
            .field("state", &self.page.state)
            .field("upload", &self.page.share)
            .finish_non_exhaustive()
    }
}

impl Transport for DryRunTransport {
    fn post(&self, _url: &str, _body: &str) -> Result<String, RequestError> {
        Ok(DRY_RUN_ANSWER.to_owned())
    }
}

impl DiagnosticsSink for Sink {
    fn note(&self, note: DiagnosticsNote) {
        self.0.send(note);
    }
}

impl Diagnostics {
    pub fn new(link: Option<DiagnosticsLink>) -> Self {
        Self {
            link,
            ..Self::default()
        }
    }

    pub const fn page(&self) -> &Page {
        &self.page
    }

    pub fn lines(&self) -> &[Vec<Run>] {
        &self.lines
    }

    /// Starts `selection`, which cancels a collection that runs.
    pub fn collect(&self, selection: Selection) {
        self.send(Request::Collect(selection));
    }

    pub fn cancel(&self) {
        self.send(Request::Cancel);
    }

    /// Uploads the redacted report the question showed. Returns whether it
    /// started: not before it is ready, without a service or while an
    /// upload runs.
    pub fn upload(&mut self) -> bool {
        let Some(text) = self
            .page
            .shareable
            .as_ref()
            .map(|ready| ready.text().to_owned())
        else {
            return false;
        };

        if self.link.is_none() || text.is_empty() || self.page.share.running() {
            return false;
        }

        self.page.start_upload();
        self.send(Request::Upload(text));
        true
    }

    pub fn clear_upload(&mut self) {
        self.page.clear_upload();
    }

    /// Takes the notes that came since the last frame. Returns what they
    /// changed, in order.
    pub fn take(&mut self) -> Vec<Changed> {
        let notes: Vec<DiagnosticsNote> = self
            .link
            .as_ref()
            .map(|link| link.notes.try_iter().collect())
            .unwrap_or_default();

        notes
            .into_iter()
            .map(|note| {
                let changed = self.page.apply(note);
                if matches!(changed, Changed::Report { ready: _ }) {
                    self.lines = if self.page.state == State::Report {
                        styled_lines(&self.page.raw)
                    } else {
                        Vec::new()
                    };
                }

                changed
            })
            .collect()
    }

    fn send(&self, request: Request) {
        if let Some(link) = &self.link {
            link.handle.send(request);
        }
    }
}

#[cfg(any(test, feature = "testing"))]
impl Diagnostics {
    /// The state a page test shows, without a service.
    pub fn showing(page: Page) -> Self {
        Self {
            lines: styled_lines(&page.raw),
            page,
            link: None,
        }
    }
}

/// Starts the service on `runtime`. Each note calls `wake`.
#[must_use = "the link is the only way to the service"]
pub fn spawn(
    runtime: &tokio::runtime::Handle,
    wake: Wake,
    source: Arc<dyn DiagnosticSource>,
    transport: Arc<dyn Transport>,
) -> DiagnosticsLink {
    let (sender, notes) = waking::channel(wake);
    let sink = Arc::new(Sink(sender));

    DiagnosticsLink {
        handle: service::spawn(runtime, source, transport, sink),
        notes,
    }
}
