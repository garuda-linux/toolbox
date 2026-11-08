//! One root segment: the helper started through the launcher, the request
//! on its stdin, events from its stdout, answers back to its stdin.

#[cfg(test)]
mod tests;

use std::process::{ExitStatus, Stdio};

use tokio::io::{AsyncBufReadExt as _, AsyncWriteExt as _, BufReader};
use tokio::process::{Child, ChildStdin};
use tokio::task::JoinHandle;
use toolbox_core::features::console::HelperFailure;
use toolbox_plan::protocol::{Answer, Event, Mode, Request, decode, encode};
use toolbox_plan::{RootAction, Step, StepId};
use toolbox_system::pkexec::{self, Refusal};
use tracing::Span;

use super::{ApplyCommand, Launcher, Problem, Report, Reporter, Run, step_ok};

/// What the classifier of pkexec's refusals needs is in the first lines.
const KEPT_STDERR_BYTES: usize = 4096;

/// The program that would not start, so a missing pkexec says which one.
#[derive(Debug, thiserror::Error)]
#[error("{program}: {source}")]
struct NotStarted {
    program: String,
    source: std::io::Error,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Heard {
    Nothing,
    Some,
    /// `Event::Done` came: the helper ended the plan itself.
    Done,
}

pub async fn run(steps: &[Step<RootAction>], run: &mut Run<'_>, launcher: &dyn Launcher) -> bool {
    let request = Request::new(launcher.mode(), steps.to_vec());

    // Checked before the helper starts: it would wait for a line that
    // never comes.
    if let Some(step) = missing_secret(&request) {
        run.reporter
            .helper_failed(HelperFailure::MissingSecret, step.to_string());
        return false;
    }

    let mut child = match start(launcher) {
        Ok(child) => child,
        Err(error) => {
            run.reporter
                .helper_failed(HelperFailure::Start, error.to_string());
            return false;
        }
    };

    let (Some(mut stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
        run.reporter
            .helper_failed(HelperFailure::NoPipes, String::new());
        return false;
    };

    let complaints = forward_stderr(&mut child);

    send(&mut stdin, &request).await;
    send_secrets(&mut stdin, &request).await;
    if let Some(size) = run.starting_size() {
        send(&mut stdin, &size).await;
    }

    let mut lines = BufReader::new(stdout).lines();
    let mut steps_ok = true;
    let mut heard = Heard::Nothing;
    let mut step = None;

    loop {
        tokio::select! {
            line = lines.next_line() => {
                let Ok(Some(line)) = line else { break };

                let event = report_line(&line, run.reporter);
                step = step_span(step, event.as_ref(), steps);
                steps_ok &= event.as_ref().and_then(step_ok) != Some(false);
                heard = match event {
                    Some(Event::Done) => Heard::Done,
                    Some(_) | None => heard.max(Heard::Some),
                };
            }
            Some(command) = run.commands.recv() => {
                if matches!(command, ApplyCommand::Cancel) && waits_for_dialog(heard, launcher) {
                    run.reporter.report(Report::CancelWaits);
                }

                // In parts: the helper refuses an input above its limit.
                for answer in run.answer_for(command).into_iter().flat_map(Answer::parts) {
                    send(&mut stdin, &answer).await;
                }
            }
        }
    }

    // The helper cancels when its stdin closes, so it stays open until
    // stdout ended.
    drop(stdin);
    finish(&mut child, steps_ok, heard, complaints, run.reporter).await
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
            Event::Accepted | Event::Rejected { reason: _ } | Event::Output { step: _, data: _ },
        )
        | None => open,
    }
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

fn start(launcher: &dyn Launcher) -> Result<Child, NotStarted> {
    let mut command = launcher.command();
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // Reaches pkexec while it still asks. The root helper cannot be
        // signalled by the user: it ends because its stdin closes.
        .kill_on_drop(true)
        .spawn()
        .map_err(|source| NotStarted {
            program: command
                .as_std()
                .get_program()
                .to_string_lossy()
                .into_owned(),
            source,
        })
}

/// An unreadable line is none.
fn report_line(line: &str, reporter: &Reporter) -> Option<Event> {
    match decode::<Event>(line) {
        Ok(event) => {
            reporter.report(Report::Step(event.clone()));
            Some(event)
        }
        Err(error) => {
            tracing::warn!("unreadable line from the helper: {error}");
            None
        }
    }
}

async fn finish(
    child: &mut Child,
    steps_ok: bool,
    heard: Heard,
    complaints: Option<JoinHandle<String>>,
    reporter: &Reporter,
) -> bool {
    let status = match child.wait().await {
        Ok(status) => status,
        Err(error) => {
            reporter.helper_failed(HelperFailure::Lost, error.to_string());
            return false;
        }
    };

    if heard == Heard::Nothing
        && let Some((refusal, words)) = refusal(status, complaints).await
    {
        if refusal == Refusal::Broken {
            let detail = pkexec::reason(&words).to_owned();
            reporter.helper_failed(HelperFailure::Pkexec, detail);
            return false;
        }

        if refusal.not_authorized() {
            tracing::info!("the plan did not start: {refusal}");
        } else {
            tracing::error!("the plan did not start: {refusal}");
        }

        reporter.report(Report::Problem(Problem::Refused(refusal)));
        return false;
    }

    // Killed or crashed: without this the plan would fail without a word.
    if heard != Heard::Done {
        reporter.helper_failed(HelperFailure::Ended, status.to_string());
        return false;
    }

    steps_ok && status.success()
}

/// Why pkexec did not run the helper, when its exit code says so. Its
/// words are waited for only then: pkexec has ended and nothing it
/// started holds the pipe.
async fn refusal(
    status: ExitStatus,
    complaints: Option<JoinHandle<String>>,
) -> Option<(Refusal, String)> {
    pkexec::refusal(status.code(), "")?;

    let words = match complaints {
        Some(complaints) => complaints.await.unwrap_or_default(),
        None => String::new(),
    };

    pkexec::refusal(status.code(), &words).map(|refusal| (refusal, words))
}

/// Logs the helper's stderr and keeps its first bytes for [`refusal`].
fn forward_stderr(child: &mut Child) -> Option<JoinHandle<String>> {
    let stderr = child.stderr.take()?;

    Some(tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        let mut kept = String::new();

        while let Ok(Some(line)) = lines.next_line().await {
            tracing::warn!("helper: {line}");

            if kept.len().saturating_add(line.len()) < KEPT_STDERR_BYTES {
                kept.push_str(&line);
                kept.push('\n');
            }
        }

        kept
    }))
}

/// A failed write means the helper is gone. Its exit tells the rest.
async fn send<T>(stdin: &mut ChildStdin, message: &T)
where
    T: serde::Serialize + Sync,
{
    let Ok(line) = encode(message) else {
        return;
    };

    write(stdin, line.as_bytes()).await;
}

async fn write(stdin: &mut ChildStdin, bytes: &[u8]) {
    if let Err(error) = stdin.write_all(bytes).await {
        tracing::debug!("cannot write to the helper: {error}");
    }
}

/// The secret of every step that needs one, each on its own line after
/// the request and outside its JSON, so no encoder, decoder or log on
/// either side handles it. A dry run sends none.
async fn send_secrets(stdin: &mut ChildStdin, request: &Request) {
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
