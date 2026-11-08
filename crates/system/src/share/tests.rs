use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use tokio::runtime::Handle;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use toolbox_privatebin::{Paste, RequestError, Transport};
use toolbox_state::diagnostics::UploadFailure;

use super::{ShareNote, ShareSink, Sharer, Upload, UploadProblem};

struct Channel(UnboundedSender<ShareNote>);

impl ShareSink for Channel {
    fn note(&self, note: ShareNote) {
        self.0.send(note).ok();
    }
}

#[derive(Default)]
struct Gate {
    open: AtomicBool,
    posts: AtomicUsize,
}

impl Transport for Gate {
    fn post(&self, _url: &str, _body: &str) -> Result<String, RequestError> {
        self.posts.fetch_add(1, Ordering::SeqCst);

        while !self.open.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(5));
        }

        Ok(r#"{"status":0,"id":"abcdef0123456789","deletetoken":"aa11"}"#.to_owned())
    }
}

fn sharer(transport: Arc<dyn Transport>) -> (Sharer, UnboundedReceiver<ShareNote>) {
    let (notes, heard) = unbounded_channel();
    let sharer = Sharer::new(Handle::current(), transport, Arc::new(Channel(notes)));
    (sharer, heard)
}

fn paste() -> Paste {
    Paste {
        link: "https://bin.garudalinux.org/?a#key".to_owned(),
        delete_link: "https://bin.garudalinux.org/?pasteid=a&deletetoken=b".to_owned(),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_share_starts_once_and_ends_with_a_link() {
    let gate = Arc::new(Gate::default());
    let (sharer, mut heard) = sharer(Arc::<Gate>::clone(&gate));

    assert!(sharer.share("the output".to_owned()));
    assert!(!sharer.share("again".to_owned()));
    assert_eq!(heard.recv().await, Some(ShareNote::Started));

    gate.open.store(true, Ordering::SeqCst);

    let Some(ShareNote::Uploaded(Ok(paste))) = heard.recv().await else {
        panic!("no paste");
    };

    assert!(
        paste
            .link
            .starts_with("https://bin.garudalinux.org/?abcdef0123456789#")
    );
    assert_eq!(gate.posts.load(Ordering::SeqCst), 1);
    assert!(sharer.share("once more".to_owned()));
}

#[test]
fn an_upload_runs_then_keeps_its_links_or_its_reason() {
    let mut upload = Upload::default();

    assert_eq!(upload, Upload::Idle);
    assert!(!upload.apply(ShareNote::Started));
    assert_eq!(upload, Upload::Running);
    assert!(upload.running());
    assert!(upload.apply(ShareNote::Uploaded(Ok(paste()))));
    assert_eq!(upload, Upload::Done(paste()));
    assert_eq!(upload.paste(), Some(&paste()));

    let problem = UploadProblem {
        kind: UploadFailure::Network,
        detail: "no route to host".to_owned(),
    };

    upload.start();
    assert_eq!(upload.paste(), None);
    assert!(!upload.apply(ShareNote::Uploaded(Err(problem.clone()))));
    assert_eq!(upload.problem(), Some(&problem));
    assert_eq!(upload, Upload::Failed(problem));

    upload.clear();
    assert_eq!(upload, Upload::Idle);
}

#[test]
fn only_a_done_upload_has_links_and_only_a_failed_one_a_reason() {
    let problem = UploadProblem::stopped();

    for upload in [Upload::Idle, Upload::Running] {
        assert_eq!((upload.paste(), upload.problem()), (None, None));
    }

    assert_eq!(Upload::Done(paste()).problem(), None);
    assert_eq!(Upload::Failed(problem).paste(), None);
}

#[test]
fn an_upload_never_prints_its_links() {
    let upload = Upload::Done(Paste {
        link: "https://bin.example/?abc#secretkey".to_owned(),
        delete_link: "https://bin.example/?pasteid=abc&deletetoken=secrettoken".to_owned(),
    });

    let printed = format!("{upload:?}");

    assert!(!printed.contains("secretkey"), "{printed}");
    assert!(!printed.contains("secrettoken"), "{printed}");
    assert!(printed.contains("Done"), "{printed}");
}
