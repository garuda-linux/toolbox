use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::{CLEAR, Spinner, drawn, frame, wanted};

#[derive(Clone, Default)]
struct Shared(Arc<parking_lot::Mutex<Vec<u8>>>);

impl Shared {
    fn text(&self) -> String {
        String::from_utf8(self.0.lock().clone()).unwrap()
    }
}

impl Write for Shared {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn the_frames_go_round() {
    assert_eq!(frame(0), '⠋');
    assert_eq!(frame(1), '⠙');
    assert_eq!(frame(9), '⠏');
    assert_eq!(frame(10), '⠋');
}

#[test]
fn a_frame_redraws_its_line_from_the_start() {
    assert_eq!(drawn(2, "Reading the system…"), "\r⠹ Reading the system…");
}

#[test]
fn it_spins_only_on_a_terminal_and_never_for_json() {
    assert!(wanted(true, false, false));
    assert!(!wanted(false, false, false));
    assert!(!wanted(true, false, true));
    assert!(!wanted(false, false, true));
}

#[test]
fn a_dumb_terminal_gets_no_spinner() {
    assert!(!wanted(true, true, false));
}

#[test]
fn a_read_done_before_the_first_frame_leaves_nothing() {
    let err = Shared::default();
    let spinner = Spinner::on(err.clone(), "Reading", Duration::from_secs(60));

    drop(spinner);

    assert_eq!(err.text(), "");
}

#[test]
fn a_drawn_spinner_is_cleared_when_it_goes() {
    let err = Shared::default();
    let spinner = Spinner::on(err.clone(), "Reading", Duration::ZERO);

    let deadline = Instant::now() + Duration::from_secs(5);
    while err.text().is_empty() && Instant::now() < deadline {
        std::thread::yield_now();
    }

    drop(spinner);

    let text = err.text();
    assert!(text.starts_with("\r⠋ Reading"), "{text:?}");
    assert!(text.ends_with(CLEAR), "{text:?}");
}
