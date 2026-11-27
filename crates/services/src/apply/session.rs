//! One helper process with its request, its answers on stdin and its events
//! on stdout. The process is reaped in [`Session::close`].

mod process;
#[cfg(test)]
mod tests;
mod verdict;

use tokio::io::{AsyncBufReadExt as _, AsyncWriteExt as _, BufReader, Lines};
use tokio::net::UnixStream;
use tokio::process::{Child, ChildStdout};
use tokio::task::JoinHandle;
use toolbox_core::features::console::HelperFailure;
use toolbox_plan::protocol::{Answer, Event, Mode, Outcome, Request, decode, encode};
use toolbox_plan::{RootAction, Step, StepId};
use tracing::Span;

use self::process::{forward_stderr, spawn};
use self::verdict::{Complaints, finish};
use super::handle::Reporter;
use super::{ApplyCommand, Launcher, Report, Run};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Heard {
    Nothing,
    Some,
    /// `Event::Done` came: the helper ended the plan itself.
    Done,
}

/// What the session read or did in one turn of its loop.
enum Turn {
    Event(Event),
    Junk,
    Answered,
    Ended,
}

pub struct Session {
    child: Child,
    stdin: UnixStream,
    lines: Lines<BufReader<ChildStdout>>,
    words: Option<JoinHandle<String>>,
    steps: Vec<Step<RootAction>>,
    heard: Heard,
    steps_ok: bool,
    span: Option<Span>,
    /// The helper said `Done` or its output ended.
    ended: bool,
    /// A cancel or finish went to the helper, so no gate is passed after it.
    stopping: bool,
}

impl Session {
    /// Starts the helper with `request` and sends its secrets. `None` when
    /// it did not start, after the report.
    pub async fn start(request: &Request, run: &Run<'_>, launcher: &dyn Launcher) -> Option<Self> {
        // Checked before the helper starts: it would wait for a line that
        // never comes.
        if let Some(step) = missing_secret(request) {
            run.reporter
                .helper_failed(HelperFailure::MissingSecret, step.to_string());
            return None;
        }

        let (mut child, mut stdin) = match spawn(launcher) {
            Ok(started) => started,
            Err(error) => {
                run.reporter
                    .helper_failed(HelperFailure::Start, error.to_string());
                return None;
            }
        };

        let Some(stdout) = child.stdout.take() else {
            run.reporter
                .helper_failed(HelperFailure::NoPipes, String::new());
            return None;
        };

        let words = forward_stderr(&mut child);

        send(&mut stdin, request).await;
        send_secrets(&mut stdin, request).await;
        if let Some(size) = run.starting_size() {
            send(&mut stdin, &size).await;
        }

        Some(Self {
            child,
            stdin,
            lines: BufReader::new(stdout).lines(),
            words,
            steps: request.steps.clone(),
            heard: Heard::Nothing,
            steps_ok: true,
            span: None,
            ended: false,
            stopping: false,
        })
    }

    /// Reads the events of a root segment until its last step finished.
    /// `false` when a step failed or the helper ended first.
    pub async fn segment(
        &mut self,
        last: StepId,
        run: &mut Run<'_>,
        launcher: &dyn Launcher,
    ) -> bool {
        loop {
            // Without `Done` the helper is gone, and the close reports that.
            if self.ended {
                return self.steps_ok && self.heard == Heard::Done;
            }

            let turn = self.turn(run, launcher).await;
            if let Turn::Event(Event::Finished { step, outcome: _ }) = turn
                && step == last
            {
                return self.steps_ok;
            }
        }
    }

    /// Answers the gate before `gate` with `Proceed`, once the helper says
    /// it waits there. `false` when the helper ends first, or a cancel or
    /// finish came, which the helper answers at the gate itself.
    pub async fn proceed(
        &mut self,
        gate: StepId,
        run: &mut Run<'_>,
        launcher: &dyn Launcher,
    ) -> bool {
        loop {
            if self.ended {
                return false;
            }

            let turn = self.turn(run, launcher).await;
            if let Turn::Event(Event::Waiting { step }) = turn
                && step == gate
            {
                if self.stopping {
                    return false;
                }

                send(&mut self.stdin, &Answer::Proceed { step: gate }).await;
                return true;
            }
        }
    }

    /// Ends the helper and waits for the process. With `early`, the helper
    /// gets a cancel first, unless it already ended. `true` when the helper
    /// said `Done`, every step ended well and it exited with 0.
    pub async fn close(mut self, early: bool, run: &mut Run<'_>, launcher: &dyn Launcher) -> bool {
        // A cancel or finish sent before ends the helper on its own.
        if early && !self.ended && !self.stopping {
            self.stopping = true;
            send(&mut self.stdin, &Answer::Cancel).await;
        }

        while !self.ended {
            self.turn(run, launcher).await;
        }

        let Self {
            mut child,
            stdin,
            lines: _,
            words,
            steps: _,
            heard,
            steps_ok,
            span: _,
            ended: _,
            stopping: _,
        } = self;

        // The helper cancels when its stdin closes, so it stays open until
        // stdout ended.
        drop(stdin);
        let complaints = Complaints { launcher, words };
        finish(&mut child, steps_ok, heard, complaints, run.reporter).await
    }

    async fn turn(&mut self, run: &mut Run<'_>, launcher: &dyn Launcher) -> Turn {
        tokio::select! {
            line = self.lines.next_line() => {
                let Some(line) = readable(line) else {
                    self.ended = true;
                    return Turn::Ended;
                };

                self.heed(&line, run.reporter).await
            }
            Some(command) = run.commands.recv() => {
                if matches!(command, ApplyCommand::Cancel) && waits_for_dialog(self.heard, launcher) {
                    run.reporter.report(Report::CancelWaits);
                }

                if matches!(command, ApplyCommand::Cancel | ApplyCommand::Finish) {
                    self.stopping = true;
                }

                // In parts: the helper refuses an input above its limit.
                for answer in run.answer_for(command).into_iter().flat_map(Answer::parts) {
                    send(&mut self.stdin, &answer).await;
                }

                Turn::Answered
            }
        }
    }

    async fn heed(&mut self, line: &str, reporter: &Reporter) -> Turn {
        let event = match decode::<Event>(line) {
            Ok(event) => event,
            Err(error) => {
                tracing::warn!("unreadable line from the helper: {error}");
                self.heard = self.heard.max(Heard::Some);
                return Turn::Junk;
            }
        };

        // The Console shows no gate, the session alone waits for it.
        if !matches!(event, Event::Waiting { step: _ }) {
            reporter.step(event.clone()).await;
        }

        self.span = step_span(self.span.take(), Some(&event), &self.steps);
        self.steps_ok &= step_ok(&event) != Some(false);
        if matches!(event, Event::Done) {
            self.heard = Heard::Done;
            self.ended = true;
        } else {
            self.heard = self.heard.max(Heard::Some);
        }

        Turn::Event(event)
    }
}

/// The span of the step the helper runs now. Its steps run in another
/// process, so the span opens and closes on its events, which times each
/// step in the app's log.
fn step_span(
    open: Option<Span>,
    event: Option<&Event>,
    steps: &[Step<RootAction>],
) -> Option<Span> {
    match event {
        Some(Event::Started { step }) => steps
            .iter()
            .find(|candidate| candidate.id == *step)
            .map(toolbox_exec::step_span),
        Some(
            Event::Finished {
                step: _,
                outcome: _,
            }
            | Event::Done,
        ) => None,
        Some(
            Event::Accepted
            | Event::Rejected { reason: _ }
            | Event::Output { step: _, data: _ }
            | Event::Waiting { step: _ },
        )
        | None => open,
    }
}

const fn step_ok(event: &Event) -> Option<bool> {
    match event {
        Event::Finished { step: _, outcome } => {
            Some(matches!(outcome, Outcome::Succeeded | Outcome::Skipped))
        }
        Event::Rejected { reason: _ } => Some(false),
        Event::Accepted
        | Event::Waiting { step: _ }
        | Event::Started { step: _ }
        | Event::Output { step: _, data: _ }
        | Event::Done => None,
    }
}

/// The next line of the helper's output, `None` at its end or when it
/// cannot be read. While too much output is unread it waits, and so does
/// the read of the helper's pipe.
fn readable(line: std::io::Result<Option<String>>) -> Option<String> {
    line.inspect_err(|error| tracing::warn!("helper output cannot be read: {error}"))
        .ok()
        .flatten()
}

/// Whether pkexec may still show its dialog: it runs and the helper
/// behind it has not spoken yet.
fn waits_for_dialog(heard: Heard, launcher: &dyn Launcher) -> bool {
    heard == Heard::Nothing && launcher.mode() == Mode::Run
}

fn missing_secret(request: &Request) -> Option<StepId> {
    let needed = request.secret_steps();

    request
        .steps
        .iter()
        .find(|step| needed.contains(&step.id) && step.action.secret().is_none())
        .map(|step| step.id)
}

/// A failed write means the helper is gone. Its exit tells the rest.
async fn send<T>(stdin: &mut UnixStream, message: &T)
where
    T: serde::Serialize + Sync,
{
    let Ok(line) = encode(message).inspect_err(|error| {
        tracing::warn!("answer to the helper not sent: {error}");
    }) else {
        return;
    };

    write(stdin, line.as_bytes()).await;
}

async fn write(stdin: &mut UnixStream, bytes: &[u8]) {
    if let Err(error) = stdin.write_all(bytes).await {
        tracing::debug!("cannot write to the helper: {error}");
    }
}

/// The secret of every step that needs one, each on its own line after
/// the request and outside its JSON, so no encoder, decoder or log on
/// either side handles it. A dry run sends none.
async fn send_secrets(stdin: &mut UnixStream, request: &Request) {
    let needed = request.secret_steps();
    for step in &request.steps {
        if !needed.contains(&step.id) {
            continue;
        }

        let Some(secret) = step.action.secret() else {
            continue;
        };

        write(stdin, secret.wire_line(step.id).expose()).await;
    }
}
