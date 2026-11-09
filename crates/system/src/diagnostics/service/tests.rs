use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use parking_lot::Mutex;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use toolbox_privatebin::{RequestError, Transport};
use toolbox_state::diagnostics::{
    Collector, Known, Outcome, RedactedKind, Selection, UploadFailure,
};

use super::{DiagnosticsHandle, DiagnosticsNote, DiagnosticsSink, Request, UploadProblem, serve};
use crate::DiagnosticSource;

/// Answers every collector with its own name. Inxi blocks until it is
/// cancelled, like a program that hangs.
struct Fake;

impl DiagnosticSource for Fake {
    fn collect(&self, collector: Collector, cancel: &Arc<AtomicBool>) -> Option<Outcome> {
        match collector {
            Collector::Inxi => {
                while !cancel.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(5));
                }

                None
            }
            Collector::Health => Some(Outcome::Unavailable),
            Collector::SystemdAnalyze
            | Collector::Journal
            | Collector::PacmanLog
            | Collector::Dmesg => Some(Outcome::Report(format!("{}\n", collector.as_str()))),
        }
    }

    /// The kernel log names this host.
    fn known(&self) -> Known {
        Known {
            host: Some("dmesg".to_owned()),
            ..Known::default()
        }
    }
}

/// Never blocks, so a full report runs through.
struct Quick;

impl DiagnosticSource for Quick {
    fn collect(&self, collector: Collector, _cancel: &Arc<AtomicBool>) -> Option<Outcome> {
        match collector {
            Collector::Inxi | Collector::Health => Some(Outcome::Unavailable),
            Collector::SystemdAnalyze
            | Collector::Journal
            | Collector::PacmanLog
            | Collector::Dmesg => Some(Outcome::Report(format!("{}\n", collector.as_str()))),
        }
    }
}

struct Channel(UnboundedSender<DiagnosticsNote>);

impl DiagnosticsSink for Channel {
    fn note(&self, note: DiagnosticsNote) {
        self.0.send(note).ok();
    }
}

/// Keeps the bodies it was sent and answers with `answer`.
struct Server {
    answer: Result<String, RequestError>,
    bodies: Mutex<Vec<String>>,
}

impl Transport for Server {
    fn post(&self, _url: &str, body: &str) -> Result<String, RequestError> {
        self.bodies.lock().push(body.to_owned());
        self.answer.clone()
    }
}

/// Takes every upload and answers with the same paste.
struct Accepts;

impl Transport for Accepts {
    fn post(&self, _url: &str, _body: &str) -> Result<String, RequestError> {
        Ok(r#"{"status":0,"id":"dryrun0000000000","deletetoken":"dryrun"}"#.to_owned())
    }
}

struct Panics;

/// Holds every upload until `open` is set, like a slow server.
#[derive(Default)]
struct Gate {
    open: AtomicBool,
    posts: AtomicUsize,
}

impl DiagnosticSource for Panics {
    fn collect(&self, _collector: Collector, _cancel: &Arc<AtomicBool>) -> Option<Outcome> {
        panic!("the collector panicked");
    }
}

impl Transport for Gate {
    fn post(&self, url: &str, body: &str) -> Result<String, RequestError> {
        self.posts.fetch_add(1, Ordering::SeqCst);

        while !self.open.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(5));
        }

        Accepts.post(url, body)
    }
}

fn start<S>(
    source: S,
    transport: Arc<dyn Transport>,
) -> (DiagnosticsHandle, UnboundedReceiver<DiagnosticsNote>)
where
    S: DiagnosticSource + 'static,
{
    let (commands, receiver) = unbounded_channel();
    let (notes, heard) = unbounded_channel();

    tokio::spawn(serve(
        receiver,
        Arc::new(source),
        transport,
        Arc::new(Channel(notes)),
        Handle::current(),
    ));

    (DiagnosticsHandle { commands }, heard)
}

fn one(collector: Collector) -> Request {
    Request::Collect(Selection::One(collector))
}

#[tokio::test(flavor = "multi_thread")]
async fn a_collector_reports_its_start_and_its_outcome() {
    let (handle, mut heard) = start(Fake, Arc::new(Accepts));

    handle.send(one(Collector::Dmesg));

    assert_eq!(
        heard.recv().await,
        Some(DiagnosticsNote::Started(Selection::One(Collector::Dmesg)))
    );
    assert_eq!(
        heard.recv().await,
        Some(DiagnosticsNote::Finished(
            Selection::One(Collector::Dmesg),
            Outcome::Report("dmesg\n".to_owned())
        ))
    );

    let Some(DiagnosticsNote::Shareable(shareable)) = heard.recv().await else {
        panic!("the redacted report follows the report");
    };

    assert_eq!(shareable.text(), "<host>\n");
    assert_eq!(shareable.count(RedactedKind::Host), 1);

    handle.send(one(Collector::Health));
    heard.recv().await;

    assert_eq!(
        heard.recv().await,
        Some(DiagnosticsNote::Finished(
            Selection::One(Collector::Health),
            Outcome::Unavailable
        ))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_full_report_names_each_collector_as_it_runs() {
    let (handle, mut heard) = start(Quick, Arc::new(Accepts));

    handle.send(Request::Collect(Selection::All));

    assert_eq!(
        heard.recv().await,
        Some(DiagnosticsNote::Started(Selection::All))
    );

    for collector in Collector::ALL {
        assert_eq!(
            heard.recv().await,
            Some(DiagnosticsNote::Collecting(collector))
        );
    }

    let Some(DiagnosticsNote::Finished(Selection::All, Outcome::Report(report))) =
        heard.recv().await
    else {
        panic!("no full report");
    };

    assert!(
        report.starts_with("### inxi ###\nNot available on this system.\n### END inxi ###\n\n")
    );
    assert!(report.contains("### dmesg ###\ndmesg\n### END dmesg ###\n"));
    assert!(report.ends_with("### END garuda-health ###\n\n"));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_new_collection_cancels_the_running_one() {
    let (handle, mut heard) = start(Fake, Arc::new(Accepts));

    handle.send(one(Collector::Inxi));
    handle.send(one(Collector::Journal));

    assert_eq!(
        heard.recv().await,
        Some(DiagnosticsNote::Started(Selection::One(Collector::Inxi)))
    );
    assert_eq!(
        heard.recv().await,
        Some(DiagnosticsNote::Started(Selection::One(Collector::Journal)))
    );

    // Nothing of the cancelled run follows, not even "cancelled": the
    // page already shows the new one.
    assert_eq!(
        heard.recv().await,
        Some(DiagnosticsNote::Finished(
            Selection::One(Collector::Journal),
            Outcome::Report("journalctl\n".to_owned())
        ))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn cancel_stops_the_run_and_says_so_once() {
    let (handle, mut heard) = start(Fake, Arc::new(Accepts));

    handle.send(Request::Cancel);
    handle.send(one(Collector::Inxi));
    handle.send(Request::Cancel);
    handle.send(Request::Cancel);
    handle.send(one(Collector::Dmesg));

    assert_eq!(
        heard.recv().await,
        Some(DiagnosticsNote::Started(Selection::One(Collector::Inxi)))
    );
    assert_eq!(heard.recv().await, Some(DiagnosticsNote::Cancelled));
    assert_eq!(
        heard.recv().await,
        Some(DiagnosticsNote::Started(Selection::One(Collector::Dmesg)))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_cancelled_full_report_stops_at_the_blocked_collector() {
    let (handle, mut heard) = start(Fake, Arc::new(Accepts));

    handle.send(Request::Collect(Selection::All));

    assert_eq!(
        heard.recv().await,
        Some(DiagnosticsNote::Started(Selection::All))
    );
    assert_eq!(
        heard.recv().await,
        Some(DiagnosticsNote::Collecting(Collector::Inxi))
    );

    handle.send(Request::Cancel);

    assert_eq!(heard.recv().await, Some(DiagnosticsNote::Cancelled));

    handle.send(one(Collector::Dmesg));

    assert_eq!(
        heard.recv().await,
        Some(DiagnosticsNote::Started(Selection::One(Collector::Dmesg)))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn an_upload_sends_the_text_encrypted_and_reports_the_link() {
    let server = Arc::new(Server {
        answer: Ok(r#"{"status":0,"id":"abcdef0123456789","deletetoken":"aa11"}"#.to_owned()),
        bodies: Mutex::new(Vec::new()),
    });

    let (handle, mut heard) = start(Fake, Arc::<Server>::clone(&server));

    handle.send(Request::Upload("the secret report".to_owned()));

    assert_eq!(heard.recv().await, Some(DiagnosticsNote::UploadStarted));

    let Some(DiagnosticsNote::Uploaded(Ok(paste))) = heard.recv().await else {
        panic!("no paste");
    };

    assert!(
        paste
            .link
            .starts_with("https://bin.garudalinux.org/?abcdef0123456789#")
    );
    assert_eq!(
        paste.delete_link,
        "https://bin.garudalinux.org/?pasteid=abcdef0123456789&deletetoken=aa11"
    );

    let bodies = server.bodies.lock().clone();

    assert_eq!(bodies.len(), 1);
    assert!(!bodies[0].contains("secret report"));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_second_upload_while_one_runs_is_refused() {
    let gate = Arc::new(Gate::default());
    let (handle, mut heard) = start(Fake, Arc::<Gate>::clone(&gate));

    handle.send(Request::Upload("first".to_owned()));
    handle.send(Request::Upload("second".to_owned()));

    assert_eq!(heard.recv().await, Some(DiagnosticsNote::UploadStarted));

    gate.open.store(true, Ordering::SeqCst);

    assert!(matches!(
        heard.recv().await,
        Some(DiagnosticsNote::Uploaded(Ok(_)))
    ));
    assert_eq!(gate.posts.load(Ordering::SeqCst), 1);

    handle.send(Request::Upload("third".to_owned()));

    assert_eq!(heard.recv().await, Some(DiagnosticsNote::UploadStarted));
    assert!(matches!(
        heard.recv().await,
        Some(DiagnosticsNote::Uploaded(Ok(_)))
    ));
    assert_eq!(gate.posts.load(Ordering::SeqCst), 2);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_failed_upload_reports_the_reason() {
    let server = Arc::new(Server {
        answer: Err(RequestError::NoAnswer("no route to host".to_owned())),
        bodies: Mutex::new(Vec::new()),
    });

    let (handle, mut heard) = start(Fake, server);

    handle.send(Request::Upload("report".to_owned()));
    heard.recv().await;

    assert_eq!(
        heard.recv().await,
        Some(DiagnosticsNote::Uploaded(Err(UploadProblem {
            kind: UploadFailure::Network,
            detail: "no route to host".to_owned(),
        })))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_panicking_collector_ends_as_a_failed_collection() {
    let (handle, mut heard) = start(Panics, Arc::new(Accepts));

    handle.send(one(Collector::Dmesg));
    assert_eq!(
        heard.recv().await,
        Some(DiagnosticsNote::Started(Selection::One(Collector::Dmesg)))
    );

    let Some(DiagnosticsNote::Finished(_, Outcome::Failed(reason))) =
        tokio::time::timeout(Duration::from_secs(20), heard.recv())
            .await
            .unwrap()
    else {
        panic!("a panic left the collection running forever");
    };

    assert_ne!(reason, "");
}
