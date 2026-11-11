//! Collects diagnostic reports and uploads them, one collection at a time.
//!
//! A new collection cancels the one that runs. The programs and the HTTPS
//! request block, so both run on blocking threads and answer through a
//! [`DiagnosticsSink`].

#[cfg(test)]
mod tests;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::runtime::Handle;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use toolbox_privatebin::{Paste, Transport};
use toolbox_pty::strip_ansi;
use toolbox_state::diagnostics::{Collector, Outcome, Redacted, Selection, full_report, redact};
use toolbox_system::DiagnosticSource;

use crate::share::{ShareNote, ShareSink, Sharer, UploadProblem};

#[derive(Debug)]
pub enum Request {
    Collect(Selection),
    Cancel,
    /// The text the confirm dialog showed.
    Upload(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticsNote {
    Started(Selection),
    Collecting(Collector),
    Finished(Selection, Outcome),
    /// The finished report as it may leave the machine. Comes right
    /// after its `Finished`.
    Shareable(Redacted),
    Cancelled,
    UploadStarted,
    Uploaded(Result<Paste, UploadProblem>),
}

pub trait DiagnosticsSink: Send + Sync {
    fn note(&self, note: DiagnosticsNote);
}

#[derive(Clone)]
pub struct DiagnosticsHandle {
    commands: UnboundedSender<Request>,
}

struct Run {
    id: u64,
    cancel: Arc<AtomicBool>,
}

struct Done {
    id: u64,
    selection: Selection,
    /// `None` after a cancel.
    outcome: Option<Outcome>,
    shareable: Option<Redacted>,
}

struct Service {
    source: Arc<dyn DiagnosticSource>,
    sink: Arc<dyn DiagnosticsSink>,
    runtime: Handle,
    results: UnboundedSender<Done>,
    run: Option<Run>,
    last_id: u64,
    /// Refuses a second upload while one runs.
    sharer: Sharer,
}

struct Relay(Arc<dyn DiagnosticsSink>);

impl ShareSink for Relay {
    fn note(&self, note: ShareNote) {
        self.0.note(match note {
            ShareNote::Started => DiagnosticsNote::UploadStarted,
            ShareNote::Uploaded(result) => DiagnosticsNote::Uploaded(result),
        });
    }
}

impl DiagnosticsHandle {
    /// Sending fails only after shutdown, when nobody listens anymore.
    pub fn send(&self, command: Request) {
        if self.commands.send(command).is_err() {
            tracing::debug!("the diagnostics service is gone");
        }
    }
}

impl Service {
    fn handle(&mut self, command: Request) {
        match command {
            Request::Collect(selection) => self.collect(selection),
            Request::Cancel => {
                if self.stop() {
                    tracing::info!("the diagnostics collection was cancelled");
                    self.sink.note(DiagnosticsNote::Cancelled);
                }
            }
            Request::Upload(text) => {
                self.sharer.share(text);
            }
        }
    }

    fn collect(&mut self, selection: Selection) {
        self.stop();
        self.last_id = self.last_id.wrapping_add(1);

        let id = self.last_id;
        let cancel = Arc::new(AtomicBool::new(false));
        let source = Arc::clone(&self.source);
        let sink = Arc::clone(&self.sink);
        let flag = Arc::clone(&cancel);

        tracing::info!("collecting diagnostics: {}", selection.as_str());
        self.sink.note(DiagnosticsNote::Started(selection));
        self.run = Some(Run { id, cancel });

        let work = self.runtime.spawn_blocking(move || {
            let outcome = run(selection, source.as_ref(), &flag, sink.as_ref());
            let shareable = match &outcome {
                Some(Outcome::Report(text)) => Some(redact(&strip_ansi(text), &source.known())),
                Some(_) | None => None,
            };

            (outcome, shareable)
        });

        // A panic in the blocking work ends as a failed collection.
        let results = self.results.clone();

        self.runtime.spawn(async move {
            let (outcome, shareable) = work.await.unwrap_or_else(|_| {
                let failed = "the diagnostics collection stopped before it answered";

                (Some(Outcome::Failed(failed.to_owned())), None)
            });

            // The service ends with the app. Nobody waits then.
            results
                .send(Done {
                    id,
                    selection,
                    outcome,
                    shareable,
                })
                .ok();
        });
    }

    /// Cancels the running collection. Returns whether one ran.
    fn stop(&mut self) -> bool {
        self.run
            .take()
            .map(|run| run.cancel.store(true, Ordering::Relaxed))
            .is_some()
    }

    fn finish(&mut self, done: Done) {
        // A cancelled run's late result is not the page's business.
        if self.run.as_ref().is_none_or(|run| run.id != done.id) {
            return;
        }

        self.run = None;

        match done.outcome {
            Some(outcome) => {
                tracing::info!("diagnostics collected: {}", done.selection.as_str());
                self.sink
                    .note(DiagnosticsNote::Finished(done.selection, outcome));

                if let Some(shareable) = done.shareable {
                    self.sink.note(DiagnosticsNote::Shareable(shareable));
                }
            }
            None => self.sink.note(DiagnosticsNote::Cancelled),
        }
    }
}

/// Starts the service on `runtime`. A panic in it is logged, the page
/// then hears nothing more.
pub fn spawn(
    runtime: &Handle,
    source: Arc<dyn DiagnosticSource>,
    transport: Arc<dyn Transport>,
    sink: Arc<dyn DiagnosticsSink>,
) -> DiagnosticsHandle {
    let (commands, receiver) = unbounded_channel();
    let task = runtime.spawn(serve(receiver, source, transport, sink, runtime.clone()));

    drop(runtime.spawn(async move {
        if let Err(error) = task.await
            && error.is_panic()
        {
            tracing::error!("the diagnostics service stopped after a panic: {error}");
        }
    }));

    DiagnosticsHandle { commands }
}

async fn serve(
    mut commands: UnboundedReceiver<Request>,
    source: Arc<dyn DiagnosticSource>,
    transport: Arc<dyn Transport>,
    sink: Arc<dyn DiagnosticsSink>,
    runtime: Handle,
) {
    let (results, mut finished) = unbounded_channel();
    let sharer = Sharer::new(
        runtime.clone(),
        transport,
        Arc::new(Relay(Arc::clone(&sink))),
    );

    let mut service = Service {
        source,
        sink,
        runtime,
        results,
        run: None,
        last_id: 0,
        sharer,
    };

    loop {
        tokio::select! {
            command = commands.recv() => match command {
                Some(command) => service.handle(command),
                None => break,
            },
            Some(done) = finished.recv() => service.finish(done),
        }
    }

    service.stop();
}

/// `None` after a cancel.
fn run(
    selection: Selection,
    source: &dyn DiagnosticSource,
    cancel: &Arc<AtomicBool>,
    sink: &dyn DiagnosticsSink,
) -> Option<Outcome> {
    match selection {
        Selection::One(collector) => source.collect(collector, cancel),
        Selection::All => {
            let mut parts = Vec::new();

            for collector in Collector::ALL {
                if cancel.load(Ordering::Relaxed) {
                    return None;
                }

                sink.note(DiagnosticsNote::Collecting(collector));
                parts.push((collector, source.collect(collector, cancel)?));
            }

            Some(Outcome::Report(full_report(&parts)))
        }
    }
}
