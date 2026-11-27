use std::io;

use toolbox_plan::procedure::{Builtin, Command, Invocation};
use toolbox_plan::protocol::{Event, Outcome};
use toolbox_plan::{Caller, ConfigText, RootAction, SecretInput, StepId, UserAction};

use crate::QueryKind;

pub trait Procedure {
    fn procedure(&self) -> Vec<Invocation>;
}

impl Procedure for RootAction {
    fn procedure(&self) -> Vec<Invocation> {
        Self::procedure(self)
    }
}

impl Procedure for UserAction {
    fn procedure(&self) -> Vec<Invocation> {
        Self::procedure(self)
    }
}

pub trait EventSink {
    fn send(&mut self, event: Event);
}

pub trait Runner {
    fn run(&mut self, step: StepId, command: &Command, sink: &mut dyn EventSink) -> Outcome;

    /// Like [`Runner::run`], but also after a cancel, and a cancel while
    /// it runs does not stop it ([`Invocation::Always`]).
    fn run_always(&mut self, step: StepId, command: &Command, sink: &mut dyn EventSink) -> Outcome;

    /// # Errors
    ///
    /// When the command cannot start or ends in a way `kind` takes as no answer.
    fn query(&mut self, command: &Command, kind: QueryKind) -> io::Result<Vec<String>>;

    fn builtin(&mut self, step: StepId, builtin: &Builtin, sink: &mut dyn EventSink) -> Outcome;

    /// Whatever the program prints is reported with the secret taken out.
    fn feed(
        &mut self,
        step: StepId,
        command: &Command,
        input: &SecretInput,
        sink: &mut dyn EventSink,
    ) -> Outcome;

    /// Without a PTY and without a secret, so what it prints is reported as it is.
    fn check(
        &mut self,
        step: StepId,
        command: &Command,
        text: &ConfigText,
        sink: &mut dyn EventSink,
    ) -> Outcome;

    fn finish_requested(&mut self) -> bool;

    /// Reports the wait as [`Event::Waiting`] and holds until the app goes
    /// on. `false` ends the run: a cancel, a closed answer channel or the
    /// stop flag. Answers that came before the wait are not kept for it.
    fn wait_to_proceed(&mut self, step: StepId, sink: &mut dyn EventSink) -> bool;

    /// Only the helper knows the account whose built AUR packages it may install.
    fn caller(&self) -> Option<Caller> {
        None
    }
}
