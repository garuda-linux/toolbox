//! The real terminal behind the question before a change, and behind a
//! password asked without echo. Questions go to `/dev/tty`, so they show
//! even when standard output goes to a file.

#[cfg(test)]
mod tests;

use std::fs::{File, OpenOptions};
use std::io::{BufRead as _, BufReader, IsTerminal as _, Read, Write as _};

use rustix::termios::{
    LocalModes, OptionalActions, SpecialCodeIndex, Termios, tcgetattr, tcsetattr,
};
use toolbox_cli::Exit;
use toolbox_cli::confirm::Prompt;
use toolbox_cli::output::{ColorMode, Paint};
use toolbox_plan::MAX_PASSWORD_BYTES;

/// Standard input and output for the question before a change: both must
/// be a terminal, an answer piped in could not have read the question.
#[derive(Clone, Copy, Debug)]
pub struct Terminal {
    /// `--color`, for the question.
    pub color: ColorMode,
}

impl Prompt for Terminal {
    fn is_terminal(&self) -> bool {
        std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
    }

    fn ask(&mut self, question: &str) -> Option<String> {
        let mut tty = tty()?;

        // A blank line sets the question apart from the plan above it. The
        // terminal is one by definition, the flags and NO_COLOR still count.
        let paint = Paint::new(super::out::style_on(self.color, true));
        write!(tty, "\n{} ", paint.question(question.trim_end())).ok()?;
        tty.flush().ok()?;

        let mut answer = String::new();
        let read = BufReader::new(tty).read_line(&mut answer).ok()?;
        (read > 0).then_some(answer)
    }

    fn secret(&mut self, question: &str) -> Option<String> {
        let mut tty = tty()?;
        let quiet = Quiet::on(&tty, Wait::ForInput)?;

        write!(tty, "{question}").ok();
        tty.flush().ok();
        let typed = read_secret(&mut tty);
        writeln!(tty).ok();
        drop(quiet);

        match typed {
            Typed::Line(answer) => Some(answer),
            Typed::Closed => None,
            Typed::Cancelled => std::process::exit(i32::from(Exit::Interrupted.code())),
        }
    }

    fn line(&mut self) -> Option<String> {
        let mut answer = String::with_capacity(MAX_PASSWORD_BYTES + 2);
        let read = std::io::stdin().lock().read_line(&mut answer).ok()?;
        (read > 0).then_some(answer)
    }
}

/// The terminal without echo, line editing or Ctrl+C as a signal, until
/// dropped. Ctrl+C reaches the reader as a byte instead, so no signal can
/// end the program while the echo is off.
struct Quiet {
    tty: File,
    saved: Termios,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Wait {
    ForInput,
    /// A read gives up after a tenth of a second, so the caller can look
    /// whether the answer is still wanted.
    Tenth,
}

struct Waiting<'read> {
    tty: &'read File,
    wanted: &'read dyn Fn() -> bool,
}

impl Read for Waiting<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            let read = self.tty.read(buf)?;
            if read > 0 || !(self.wanted)() {
                return Ok(read);
            }
        }
    }
}

impl Quiet {
    fn on(tty: &File, wait: Wait) -> Option<Self> {
        let saved = tcgetattr(tty).ok()?;
        let mut quiet = saved.clone();
        quiet
            .local_modes
            .remove(LocalModes::ECHO | LocalModes::ICANON | LocalModes::ISIG);
        let (least, tenths) = match wait {
            Wait::ForInput => (1, 0),
            Wait::Tenth => (0, 1),
        };

        quiet.special_codes[SpecialCodeIndex::VMIN] = least;
        quiet.special_codes[SpecialCodeIndex::VTIME] = tenths;

        // Flush drops what was typed ahead, so it cannot become the secret.
        tcsetattr(tty, OptionalActions::Flush, &quiet).ok()?;

        Some(Self {
            tty: tty.try_clone().ok()?,
            saved,
        })
    }
}

impl Drop for Quiet {
    fn drop(&mut self) {
        if let Err(error) = tcsetattr(&self.tty, OptionalActions::Now, &self.saved) {
            tracing::warn!("the terminal's echo could not be turned back on: {error}");
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Typed {
    Line(String),
    Cancelled,
    /// Ctrl+D on an empty line, or the input ended before Return.
    Closed,
}

/// The program's question answered while a plan runs: one line typed
/// without echo, read while `wanted` says so.
pub fn hidden_line(wanted: &dyn Fn() -> bool) -> Option<Typed> {
    let tty = tty()?;
    let quiet = Quiet::on(&tty, Wait::Tenth)?;
    let mut waiting = Waiting { tty: &tty, wanted };
    let typed = read_secret(&mut waiting);
    writeln!(&tty).ok();
    drop(quiet);

    Some(typed)
}

fn tty() -> Option<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .ok()
}

/// Reads one secret line byte by byte from a terminal without line
/// editing: Return or a line feed ends it, Backspace takes back the last
/// character. A secret cut short is overwritten before it is dropped.
pub fn read_secret(input: &mut dyn Read) -> Typed {
    let mut typed: Vec<u8> = Vec::with_capacity(MAX_PASSWORD_BYTES + 1);
    let mut byte = [0_u8];

    let ended = loop {
        match input.read(&mut byte) {
            Ok(1) => {}
            Ok(_) | Err(_) => break Typed::Closed,
        }

        match byte {
            [b'\r' | b'\n'] => {
                break match String::from_utf8(std::mem::take(&mut typed)) {
                    Ok(line) => Typed::Line(line),
                    Err(error) => {
                        wipe_bytes(error.into_bytes());
                        Typed::Closed
                    }
                };
            }
            [0x03] => break Typed::Cancelled,
            [0x04] if typed.is_empty() => break Typed::Closed,
            [0x7f | 0x08] => {
                // A whole character goes, not the last byte of it.
                while let Some(last) = typed.pop() {
                    if last & 0b1100_0000 != 0b1000_0000 {
                        break;
                    }
                }
            }
            [other] if typed.len() < typed.capacity() => typed.push(other),
            [_] => {}
        }
    };

    wipe_bytes(typed);
    ended
}

/// The line without its line break, in place, so no copy of a secret is
/// left behind.
pub fn without_line_end(mut line: String) -> String {
    let kept = line.trim_end_matches(['\n', '\r']).len();
    line.truncate(kept);
    line
}

pub fn wipe(text: impl Into<Vec<u8>>) {
    wipe_bytes(text.into());
}

fn wipe_bytes(mut bytes: Vec<u8>) {
    bytes.fill(0);

    // Without a reader the optimiser may drop the fill of a buffer that is
    // freed right after.
    std::hint::black_box(&bytes);
}
