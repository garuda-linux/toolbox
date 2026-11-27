mod console;
mod program;
mod query;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, TryRecvError};
use std::time::{Duration, Instant};
use std::{fmt, io, thread};

pub use query::QueryKind;
use toolbox_plan::procedure::{Builtin, Command, Program};
use toolbox_plan::protocol::{Answer, Event, Outcome};
use toolbox_plan::{Caller, ConfigText, SecretInput, StepId};
use toolbox_pty::{Chunk, DEFAULT_SIZE, Exit, PtyCommand, Session, TerminalSize};

use self::program::{after, forward_rest, pty_command, report};
use crate::apply::{EventSink, Runner};
use crate::builtin::{self, SYSTEM_ROOT};
use crate::feed::feed;
use crate::report::failure;
use crate::{aur, check};

/// pkexec gives a minimal environment. Programs get only this, never
/// anything the caller could have set.
const SYSTEM_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/bin";

/// Programs print in English so their output stays parseable.
const LOCALE: &str = "C.UTF-8";

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
    /// Who started the helper, as the kernel names the peer of its stdin.
    caller: Option<Caller>,
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
            caller: None,
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

    /// The account whose built AUR packages the helper may install.
    #[must_use = "builder methods return the changed runner"]
    pub const fn for_caller(mut self, caller: Option<Caller>) -> Self {
        self.caller = caller;
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

    fn stop_requested(&self) -> bool {
        self.stop
            .as_ref()
            .is_some_and(|stop| stop.load(Ordering::Relaxed))
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
        if !self.cancelled && self.stop_requested() {
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
                (Answer::Proceed { step: _ }, _) => {
                    tracing::warn!("ignoring a proceed that no gate waits for");
                }
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

    fn cancelled_before(&mut self) -> bool {
        self.take_answers(None);
        self.cancelled
    }

    fn execute(&mut self, step: StepId, command: &Command, sink: &mut dyn EventSink) -> Outcome {
        let program = crate::located::program(command.program(), self.environment);
        let pty = pty_command(&program, command, self.size, self.environment);

        self.spawn_pty(step, command, &pty, sink)
    }

    /// Runs `pty` in a terminal and reports how it ended. `what` names the
    /// program in the errors.
    fn spawn_pty(
        &mut self,
        step: StepId,
        what: &dyn fmt::Display,
        pty: &PtyCommand,
        sink: &mut dyn EventSink,
    ) -> Outcome {
        let mut session = match Session::spawn(pty) {
            Ok(session) => session,
            Err(error) => return failure(step, sink, &format!("cannot start {what}: {error}")),
        };

        let cancelled = self.cancelled && self.guard == Guard::Cancellable;

        match self.pump(step, &mut session, sink) {
            Ok(exit) => after(step, exit, cancelled, sink),
            Err(error) => failure(step, sink, &format!("lost track of {what}: {error}")),
        }
    }
}

impl Runner for PtyRunner {
    fn run(&mut self, step: StepId, command: &Command, sink: &mut dyn EventSink) -> Outcome {
        if self.cancelled_before() {
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
    fn query(&mut self, command: &Command, kind: QueryKind) -> io::Result<Vec<String>> {
        let cancel = self.stop.clone().unwrap_or_default();

        let program = crate::located::program(command.program(), self.environment);
        query::run(program, command, kind, self.environment, &cancel)
    }

    fn builtin(&mut self, step: StepId, builtin: &Builtin, sink: &mut dyn EventSink) -> Outcome {
        if let Builtin::ReviewAur { bases } = builtin {
            let paru = crate::located::program(Program::Paru, Environment::Inherited);
            let git = crate::located::program(Program::Git, Environment::Clean);
            return aur::review::run(self, (paru.as_path(), git.as_path()), step, bases, sink);
        }

        let Builtin::BuildAur { base, run } = builtin else {
            return builtin::execute(
                Path::new(SYSTEM_ROOT),
                self.home.as_deref(),
                step,
                builtin,
                sink,
            );
        };

        let makepkg = crate::located::program(Program::Makepkg, self.environment);
        let git = crate::located::program(Program::Git, Environment::Clean);
        self.build_aur(step, (makepkg.as_path(), git.as_path()), (base, run), sink)
    }

    fn feed(
        &mut self,
        step: StepId,
        command: &Command,
        input: &SecretInput,
        sink: &mut dyn EventSink,
    ) -> Outcome {
        if self.cancelled_before() {
            return Outcome::Cancelled;
        }

        feed(step, command, self.environment, input, sink)
    }

    /// See [`check::check`]. The stop flag ends it as it ends a query.
    fn check(
        &mut self,
        step: StepId,
        command: &Command,
        text: &ConfigText,
        sink: &mut dyn EventSink,
    ) -> Outcome {
        if self.cancelled_before() {
            return Outcome::Cancelled;
        }

        let cancel = self.stop.clone().unwrap_or_default();
        let checked = check::check(command, self.environment, text, &cancel);

        check::report(step, command, check::CHECK_DEADLINE, checked, sink)
    }

    fn caller(&self) -> Option<Caller> {
        self.caller
    }

    /// Keystrokes and secrets are dropped here: no program is running that
    /// could take them.
    fn wait_to_proceed(&mut self, step: StepId, sink: &mut dyn EventSink) -> bool {
        // Answers that came before the gate are stale, so none is kept for it.
        self.take_answers(None);
        sink.send(Event::Waiting { step });

        loop {
            if self.stop_requested() {
                tracing::info!("asked to stop by a signal");
                self.cancel(None);
            }

            if self.cancelled || self.finishing {
                return false;
            }

            match self.answers.recv_timeout(POLL_INTERVAL) {
                Ok(Answer::Proceed { step: answered }) if answered == step => return true,
                Ok(Answer::Proceed { step: _ }) => {
                    tracing::warn!("ignoring a proceed that does not match the gate");
                }
                Ok(Answer::Cancel) | Err(RecvTimeoutError::Disconnected) => self.cancel(None),
                Ok(Answer::Finish) => self.finishing = true,
                Ok(Answer::Resize { size }) => {
                    self.size = TerminalSize::from((size.columns, size.rows));
                }
                Ok(Answer::Input { data: _ }) | Err(RecvTimeoutError::Timeout) => {}
            }
        }
    }

    fn finish_requested(&mut self) -> bool {
        self.take_answers(None);
        self.finishing
    }
}
