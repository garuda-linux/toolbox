mod query;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, TryRecvError};
use std::time::{Duration, Instant};
use std::{io, thread};

use toolbox_plan::procedure::{Builtin, Command};
use toolbox_plan::protocol::{Answer, Event, Outcome};
use toolbox_plan::{SecretInput, StepId};
use toolbox_pty::{Chunk, DEFAULT_SIZE, Exit, PtyCommand, Session, TerminalSize};

use crate::apply::{EventSink, Runner};
use crate::builtin::{self, SYSTEM_ROOT};
use crate::feed::feed;
use crate::report::{ended, failure, note};

/// pkexec gives a minimal environment. Programs get only this, never
/// anything the caller could have set.
pub const SYSTEM_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/bin";

/// Programs print in English so their output stays parseable.
pub const LOCALE: &str = "C.UTF-8";

const POLL_INTERVAL: Duration = Duration::from_millis(50);

const LAST_OUTPUT: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Environment {
    /// Root steps: a fixed `PATH` and locale, nothing the caller could set.
    Clean,
    /// User steps: the user's own session, so paru finds `HOME`, the
    /// editor for its diff review and the user's language.
    Inherited,
}

impl Environment {
    /// What the program gets before the variables of its command.
    pub const fn variables(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Clean => &[("PATH", SYSTEM_PATH), ("LANG", LOCALE)],
            Self::Inherited => &[],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Guard {
    Cancellable,
    /// Runs to its end even after a cancel ([`Runner::run_always`]).
    Shielded,
}

pub struct PtyRunner {
    answers: Receiver<Answer>,
    environment: Environment,
    size: TerminalSize,
    cancelled: bool,
    /// [`Answer::Finish`] came: the running program ends on its own.
    finishing: bool,
    guard: Guard,
    /// The user's home for edits of their config files, `None` in the helper.
    home: Option<PathBuf>,
    /// Set from outside to cancel like [`Answer::Cancel`]: the helper's
    /// signal handlers, which can do no more than set a flag.
    stop: Option<Arc<AtomicBool>>,
}

impl PtyRunner {
    #[must_use = "a runner does nothing until it runs steps"]
    pub const fn new(answers: Receiver<Answer>, environment: Environment) -> Self {
        Self {
            answers,
            environment,
            size: DEFAULT_SIZE,
            cancelled: false,
            finishing: false,
            guard: Guard::Cancellable,
            home: None,
            stop: None,
        }
    }

    /// Lets builtins edit config files below `home`. Only the app, running
    /// as the user, sets one.
    #[must_use = "builder methods return the changed runner"]
    pub fn with_home(mut self, home: PathBuf) -> Self {
        self.home = Some(home);
        self
    }

    /// Cancels once `stop` is set, as [`Answer::Cancel`] does. Programs run
    /// in a session of their own, so a signal for the helper never reaches
    /// them and the cleanup of a step still runs.
    #[must_use = "builder methods return the changed runner"]
    pub fn stop_on(mut self, stop: Arc<AtomicBool>) -> Self {
        self.stop = Some(stop);
        self
    }

    fn cancel(&mut self, session: Option<&mut Session>) {
        if let Some(running) = session
            && !self.cancelled
            && self.guard == Guard::Cancellable
        {
            report(running.cancel(Instant::now()), "cancel");
        }

        self.cancelled = true;
    }

    /// Applies waiting answers. A closed channel means the app is gone,
    /// which cancels: nothing should keep running as root without a UI.
    fn take_answers(&mut self, mut session: Option<&mut Session>) {
        if !self.cancelled
            && self
                .stop
                .as_ref()
                .is_some_and(|stop| stop.load(Ordering::Relaxed))
        {
            tracing::info!("asked to stop by a signal");
            self.cancel(session.as_deref_mut());
        }

        loop {
            let answer = match self.answers.try_recv() {
                Ok(answer) => answer,
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => Answer::Cancel,
            };

            match (answer, session.as_deref_mut()) {
                (Answer::Cancel, running) => {
                    let stops =
                        running.is_some() && !self.cancelled && self.guard == Guard::Cancellable;

                    self.cancel(running);

                    if !stops {
                        return;
                    }
                }
                (Answer::Input { data }, Some(running)) => {
                    report(running.input(&data), "answer");
                }
                (Answer::Resize { size }, running) => {
                    self.size = TerminalSize::from((size.columns, size.rows));

                    if let Some(running) = running {
                        report(running.resize(self.size), "resize");
                    }
                }
                (Answer::Finish, _) => self.finishing = true,
                (Answer::Input { data: _ }, None) => {}
            }
        }
    }

    fn pump(
        &mut self,
        step: StepId,
        session: &mut Session,
        sink: &mut dyn EventSink,
    ) -> io::Result<Exit> {
        let mut open = true;

        loop {
            if open {
                match session.output().recv_timeout(POLL_INTERVAL) {
                    Ok(Chunk::Text(data)) => sink.send(Event::Output { step, data }),
                    Ok(Chunk::Closed) | Err(RecvTimeoutError::Disconnected) => open = false,
                    Err(RecvTimeoutError::Timeout) => {}
                }
            } else {
                // A closed channel answers at once. A program that closed
                // its output and runs on would spin this loop.
                thread::sleep(POLL_INTERVAL);
            }

            self.take_answers(Some(session));

            if session.is_cancelled() {
                session.poll_cancel(Instant::now())?;
            }

            if let Some(exit) = session.try_wait()? {
                forward_rest(step, session, sink);
                return Ok(exit);
            }
        }
    }

    fn execute(&mut self, step: StepId, command: &Command, sink: &mut dyn EventSink) -> Outcome {
        let mut session = match Session::spawn(&pty_command(
            &crate::located::program(command.program(), self.environment),
            command,
            self.size,
            self.environment,
        )) {
            Ok(session) => session,
            Err(error) => return failure(step, sink, &format!("cannot start {command}: {error}")),
        };

        let cancelled = self.cancelled && self.guard == Guard::Cancellable;

        match self.pump(step, &mut session, sink) {
            Ok(exit) => after(step, exit, cancelled, sink),
            Err(error) => failure(step, sink, &format!("lost track of {command}: {error}")),
        }
    }
}

impl Runner for PtyRunner {
    fn run(&mut self, step: StepId, command: &Command, sink: &mut dyn EventSink) -> Outcome {
        self.take_answers(None);

        if self.cancelled {
            return Outcome::Cancelled;
        }

        self.execute(step, command, sink)
    }

    fn run_always(&mut self, step: StepId, command: &Command, sink: &mut dyn EventSink) -> Outcome {
        self.take_answers(None);
        self.guard = Guard::Shielded;

        let outcome = self.execute(step, command, sink);

        self.guard = Guard::Cancellable;
        outcome
    }

    /// See [`query::run`]. The stop flag ends it, an [`Answer::Cancel`]
    /// is only read once it returned.
    fn query(&mut self, command: &Command) -> io::Result<Vec<String>> {
        let cancel = self.stop.clone().unwrap_or_default();

        let program = crate::located::program(command.program(), self.environment);
        query::run(program, command, self.environment, &cancel)
    }

    fn builtin(&mut self, step: StepId, builtin: &Builtin, sink: &mut dyn EventSink) -> Outcome {
        builtin::execute(
            Path::new(SYSTEM_ROOT),
            self.home.as_deref(),
            step,
            builtin,
            sink,
        )
    }

    fn feed(
        &mut self,
        step: StepId,
        command: &Command,
        input: &SecretInput,
        sink: &mut dyn EventSink,
    ) -> Outcome {
        self.take_answers(None);

        if self.cancelled {
            return Outcome::Cancelled;
        }

        feed(step, command, self.environment, input, sink)
    }

    fn finish_requested(&mut self) -> bool {
        self.take_answers(None);
        self.finishing
    }
}

fn pty_command(
    program: &Path,
    command: &Command,
    size: TerminalSize,
    environment: Environment,
) -> PtyCommand {
    let base = PtyCommand::new(program)
        .args(command.arguments())
        .size(size);

    let base = match environment {
        Environment::Clean => base.env_clear(),
        Environment::Inherited => base,
    };

    environment
        .variables()
        .iter()
        .chain(command.variables())
        .fold(base, |pty, (key, value)| pty.env(key, value))
}

/// The last lines of a failing program say why. They are read until the
/// terminal closes, but a program that left a child holding it open does
/// not hold the step longer than [`LAST_OUTPUT`].
fn forward_rest(step: StepId, session: &Session, sink: &mut dyn EventSink) {
    let end = Instant::now() + LAST_OUTPUT;

    loop {
        let left = end.saturating_duration_since(Instant::now());

        match session.output().recv_timeout(left) {
            Ok(Chunk::Text(data)) => sink.send(Event::Output { step, data }),
            Ok(Chunk::Closed) | Err(RecvTimeoutError::Disconnected) => return,
            Err(RecvTimeoutError::Timeout) => {
                tracing::warn!("the terminal stayed open after the program ended");
                return;
            }
        }
    }
}

fn report(result: io::Result<()>, what: &str) {
    if let Err(error) = result {
        tracing::warn!("cannot {what} the running program: {error}");
    }
}

/// A cancel read in the same pass as a clean exit came too late: the
/// program finished its work, `pacman -S` may have committed.
fn after(step: StepId, exit: Exit, cancelled: bool, sink: &mut dyn EventSink) -> Outcome {
    match (exit, cancelled) {
        (Exit::Code(0), true) => note(
            step,
            sink,
            "the cancel came after the program had finished",
            Outcome::Succeeded,
        ),
        (_, true) => Outcome::Cancelled,
        (Exit::Code(0), false) => Outcome::Succeeded,
        (exit, false) => ended(exit),
    }
}
