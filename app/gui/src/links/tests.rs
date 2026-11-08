use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use parking_lot::Mutex;
use toolbox_gui_kit::toasts::{ToastTexts, Toasts};

use super::{Opener, install, open, open_file};
use crate::testing::PATIENCE;

const FAILED: &str = "The link did not open.";

#[derive(Default)]
struct Recording(Mutex<Option<String>>);

struct Fake {
    works: bool,
    opened: Arc<AtomicUsize>,
}

impl Opener for Fake {
    fn open(&self, _url: &str) -> bool {
        self.opened.fetch_add(1, Ordering::SeqCst);
        self.works
    }
}

impl Opener for Recording {
    fn open(&self, url: &str) -> bool {
        *self.0.lock() = Some(url.to_owned());
        true
    }
}

fn window(works: bool, url: &'static str) -> (Harness<'static, Toasts>, Arc<AtomicUsize>) {
    let opened = Arc::new(AtomicUsize::new(0));
    let fake = Fake {
        works,
        opened: Arc::clone(&opened),
    };

    let harness = toolbox_gui_kit::testing::harness(600.0, 300.0).build_ui_state(
        |ui, toasts: &mut Toasts| {
            let texts = ToastTexts {
                name: "Notification",
                close: "Close",
            };

            toasts.show(ui.ctx(), ui.max_rect(), texts);
        },
        Toasts::default(),
    );

    install(&harness.ctx, Arc::new(fake));
    open(&harness.ctx, url);
    (harness, opened)
}

fn wait_for_note(harness: &mut Harness<'static, Toasts>) -> bool {
    let start = Instant::now();
    while start.elapsed() < PATIENCE {
        harness.step();
        if harness.query_by_label(FAILED).is_some() {
            return true;
        }

        std::thread::sleep(Duration::from_millis(10));
    }

    false
}

#[test]
fn a_link_that_does_not_open_raises_a_note() {
    let (mut harness, opened) = window(false, "https://garudalinux.org");

    assert!(wait_for_note(&mut harness));
    assert_eq!(opened.load(Ordering::SeqCst), 1);
}

#[test]
fn a_link_that_opens_raises_nothing() {
    let (mut harness, opened) = window(true, "https://garudalinux.org");

    let start = Instant::now();
    while opened.load(Ordering::SeqCst) == 0 && start.elapsed() < PATIENCE {
        std::thread::sleep(Duration::from_millis(5));
    }

    harness.run_steps(5);
    assert!(harness.query_by_label(FAILED).is_none());
}

#[test]
fn only_web_links_reach_the_opener() {
    let (mut harness, opened) = window(true, "--help");

    assert!(wait_for_note(&mut harness), "refused, and told so");
    assert_eq!(opened.load(Ordering::SeqCst), 0);
}

#[test]
fn a_file_opens_as_a_file_link() {
    let recording = Arc::new(Recording::default());
    let ctx = egui::Context::default();
    install(&ctx, Arc::<Recording>::clone(&recording));

    open_file(&ctx, Path::new("/home/ada/my notes.txt"));

    let start = Instant::now();
    let opened = loop {
        let opened = recording.0.lock().clone();

        if opened.is_some() || start.elapsed() > PATIENCE {
            break opened;
        }

        std::thread::sleep(Duration::from_millis(5));
    };

    assert_eq!(opened.as_deref(), Some("file:///home/ada/my%20notes.txt"));
}
