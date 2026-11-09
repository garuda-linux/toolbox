#[cfg(test)]
mod tests;

use std::io::{self, BufRead, Read as _, Write as _};
use std::process::ExitCode;
use std::sync::mpsc::{self, Receiver};

use toolbox_exec::EventSink;
use toolbox_plan::protocol::{
    Answer, Event, MAX_LINE_BYTES, Request, RequestError, decode, encode,
};
use toolbox_plan::{MAX_PASSWORD_BYTES, parse_secret_line};

/// Room for `secret <step> ` and the newline around a password.
const SECRET_LINE_OVERHEAD: usize = 32;

const USAGE_EXIT: u8 = 2;

/// A full queue holds the reader back, so the app waits and the root
/// helper's memory stays bounded whatever the app sends.
pub const ANSWER_QUEUE: usize = 64;

/// Flushed per line, so the app sees output as it happens.
pub struct StdoutSink;

impl EventSink for StdoutSink {
    fn send(&mut self, event: Event) {
        let written = encode(&event)
            .map_err(io::Error::other)
            .and_then(|line| print_line(&line));

        if let Err(error) = written {
            note(&format!("cannot report to the app: {error}"));
        }
    }
}

/// # Errors
///
/// [`RequestError`] for a missing, overlong, malformed or invalid request.
pub fn read_request<R>(input: &mut R) -> Result<Request, RequestError>
where
    R: BufRead,
{
    let line = read_line(input)
        .map_err(|error| RequestError::Malformed(error.to_string()))?
        .ok_or_else(|| RequestError::Malformed("no request on stdin".to_owned()))?;

    Request::parse(&line)
}

/// The lines follow the request and are no JSON, so no decoder or log ever sees them.
///
/// # Errors
///
/// [`RequestError::MissingSecret`] naming the step whose line is missing,
/// malformed or meant for another step. The line itself is never quoted.
pub fn read_secrets<R>(input: &mut R, request: &mut Request) -> Result<(), RequestError>
where
    R: BufRead,
{
    for step in request.secret_steps() {
        let missing = RequestError::MissingSecret(step);
        let line = read_secret_line(input).ok_or_else(|| missing.clone())?;
        let (named, secret) = parse_secret_line(line).map_err(|_malformed| missing.clone())?;

        if named != step {
            return Err(missing);
        }

        request.attach_secret(step, secret)?;
    }

    Ok(())
}

/// One raw line, newline included, `None` when it is missing, cut off or
/// longer than any secret line can be. What was read is overwritten then.
/// A rejected line is drained to the next newline, so its tail never
/// becomes the next step's secret.
fn read_secret_line<R>(input: &mut R) -> Option<Vec<u8>>
where
    R: BufRead,
{
    let limit =
        u64::try_from(MAX_PASSWORD_BYTES.saturating_add(SECRET_LINE_OVERHEAD)).unwrap_or(u64::MAX);

    let mut line = Vec::new();
    let read = input.by_ref().take(limit).read_until(b'\n', &mut line);

    if read.is_ok() && line.ends_with(b"\n") {
        Some(line)
    } else {
        line.fill(0);
        drain_line(input);
        None
    }
}

/// Discards bytes to the next newline or the end of input, overwriting
/// the buffer, so an overlong line leaves nothing for the next read.
fn drain_line<R>(input: &mut R)
where
    R: BufRead,
{
    let mut discard = Vec::new();

    loop {
        discard.clear();

        match input.read_until(b'\n', &mut discard) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let done = discard.ends_with(b"\n");
                discard.fill(0);

                if done {
                    break;
                }
            }
        }
    }
}

/// Reads on a thread until stdin closes. The closed channel tells the runner
/// the app is gone.
pub fn spawn_answer_reader<R>(mut input: R) -> Receiver<Answer>
where
    R: BufRead + Send + 'static,
{
    let (sender, receiver) = mpsc::sync_channel(ANSWER_QUEUE);

    let reader = std::thread::Builder::new()
        .name("answers".to_owned())
        .spawn(move || {
            while let Ok(Some(line)) = read_line(&mut input) {
                match decode::<Answer>(&line) {
                    Ok(answer) => {
                        if sender.send(answer).is_err() {
                            return;
                        }
                    }
                    // Without the decoder's message: it may quote what it read.
                    Err(_) => note("ignoring an unreadable answer"),
                }
            }
        });

    // Without the thread the channel is closed at once, so the runner
    // stops as it does when the app goes away.
    if reader.is_err() {
        note("cannot start the thread that reads answers");
    }

    receiver
}

/// Longer lines than the protocol allows are an error, never buffered whole.
fn read_line<R>(input: &mut R) -> io::Result<Option<String>>
where
    R: BufRead,
{
    let mut line = String::new();
    let limit = u64::try_from(MAX_LINE_BYTES)
        .unwrap_or(u64::MAX)
        .saturating_add(1);

    let read = input.by_ref().take(limit).read_line(&mut line)?;

    if read == 0 {
        return Ok(None);
    }

    if line.len() > MAX_LINE_BYTES {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "line too long"));
    }

    Ok(Some(line.trim_end_matches(['\n', '\r']).to_owned()))
}

/// # Errors
///
/// When stdout is closed.
pub fn print_line(line: &str) -> io::Result<()> {
    let mut stdout = io::stdout().lock();

    stdout.write_all(line.as_bytes())?;
    stdout.flush()
}

/// Prints the answer line of a read, or [`note`]s why there is none
/// (`what` names the read) and fails.
pub fn print_answer(answer: io::Result<String>, what: &str) -> ExitCode {
    match answer.and_then(|line| print_line(&line)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            note(&format!("{what}: {error}"));
            ExitCode::FAILURE
        }
    }
}

/// A line on stderr, which the app adds to its log. Root has no log file.
pub fn note(message: &str) {
    // Nothing else could report a broken stderr.
    writeln!(io::stderr(), "garuda-toolbox-helper: {message}").ok();
}

pub fn refuse(message: &str) -> ExitCode {
    note(message);
    ExitCode::from(USAGE_EXIT)
}
