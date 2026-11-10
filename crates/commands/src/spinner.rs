//! A spinner on standard error while a slow read runs, so a terminal shows
//! that something happens. Standard output stays the result alone.

#[cfg(test)]
mod tests;

use std::io::{IsTerminal as _, Write};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::thread::JoinHandle;
use std::time::Duration;

const FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// The time between frames, and before the first one, so a fast read
/// never flashes a spinner.
const PACE: Duration = Duration::from_millis(80);

const READING: &str = "Reading the system…";

const CLEAR: &str = "\r\u{1b}[2K";

pub struct Spinner {
    stop: Option<Sender<()>>,
    drawing: Option<JoinHandle<()>>,
}

impl Spinner {
    fn on(out: impl Write + Send + 'static, text: &'static str, first: Duration) -> Self {
        let (stop, stopped) = channel::<()>();
        let started = std::thread::Builder::new()
            .name("cli-spinner".to_owned())
            .spawn(move || spin(out, text, first, &stopped));

        let drawing = started
            .inspect_err(|error| tracing::warn!("no spinner while reading: {error}"))
            .ok();

        Self {
            stop: Some(stop),
            drawing,
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.stop.take();
        if let Some(drawing) = self.drawing.take()
            && drawing.join().is_err()
        {
            tracing::warn!("the spinner stopped unexpectedly");
        }
    }
}

/// `read` behind a spinner, gone before the result prints. `quiet`, as
/// for JSON output, `--quiet`, standard error that is no terminal or a
/// `TERM=dumb` one, which cannot clear a line, show none.
pub fn while_reading<T>(quiet: bool, read: impl FnOnce() -> T) -> T {
    while_working(quiet, READING, read)
}

/// [`while_reading`] with its own `text`.
pub fn while_working<T>(quiet: bool, text: &'static str, work: impl FnOnce() -> T) -> T {
    let spinner = wanted(
        std::io::stderr().is_terminal(),
        std::env::var_os("TERM").is_some_and(|term| term == "dumb"),
        quiet,
    )
    .then(|| Spinner::on(std::io::stderr(), text, PACE));

    let found = work();
    drop(spinner);
    found
}

const fn wanted(terminal: bool, dumb: bool, quiet: bool) -> bool {
    terminal && !dumb && !quiet
}

fn frame(tick: usize) -> char {
    FRAMES.get(tick % FRAMES.len()).copied().unwrap_or(' ')
}

fn drawn(tick: usize, text: &str) -> String {
    format!("\r{} {text}", frame(tick))
}

fn spin(mut out: impl Write, text: &str, first: Duration, stopped: &Receiver<()>) {
    let mut wait = first;
    let mut tick = 0;

    while stopped.recv_timeout(wait) == Err(RecvTimeoutError::Timeout) {
        out.write_all(drawn(tick, text).as_bytes()).ok();
        out.flush().ok();
        tick += 1;
        wait = PACE;
    }

    if tick > 0 {
        out.write_all(CLEAR.as_bytes()).ok();
        out.flush().ok();
    }
}
