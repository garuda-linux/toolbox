use std::collections::VecDeque;
use std::io;

use toolbox_plan::procedure::{Builtin, Command, Program};
use toolbox_plan::protocol::{Failure, Outcome};
use toolbox_plan::{EditCopy, Editor, SecretInput, StepId, UserAction};

use super::super::{Procedure, run_once, with_targets};
use super::FakeRunner;
use crate::apply::{EventSink, Runner};

#[test]
fn user_actions_list_their_invocations() {
    let action = UserAction::EditFile {
        editor: Editor::Micro,
        copy: EditCopy::new("/tmp/garuda-toolbox/edit/copy").unwrap(),
    };

    assert_eq!(action.procedure().len(), 1);
    assert_eq!(
        <UserAction as Procedure>::procedure(&action).len(),
        1,
        "the runner asks the trait, not the action"
    );
}

#[test]
fn an_always_step_runs_through_run_always() {
    use toolbox_plan::procedure::Invocation;

    let mut runner = FakeRunner {
        outcomes: VecDeque::from([Outcome::Succeeded]),
        ..FakeRunner::default()
    };

    let mut events = Vec::new();

    let command = Command::new(Program::Mount);

    let outcome = run_once(
        StepId::new(1),
        &Invocation::Always(command),
        &mut runner,
        &mut events,
    );

    assert_eq!(outcome, Outcome::Succeeded);
    assert!(runner.ran[0].starts_with("always: "));
}

#[test]
fn a_failing_query_fails_its_step() {
    struct QueryFails;

    impl Runner for QueryFails {
        fn run(&mut self, _step: StepId, _command: &Command, _sink: &mut dyn EventSink) -> Outcome {
            Outcome::Succeeded
        }

        fn run_always(
            &mut self,
            _step: StepId,
            _command: &Command,
            _sink: &mut dyn EventSink,
        ) -> Outcome {
            Outcome::Succeeded
        }

        fn query(&mut self, _command: &Command) -> io::Result<Vec<String>> {
            Err(io::Error::other("pacman is not installed"))
        }

        fn finish_requested(&mut self) -> bool {
            false
        }

        fn builtin(
            &mut self,
            _step: StepId,
            _builtin: &Builtin,
            _sink: &mut dyn EventSink,
        ) -> Outcome {
            Outcome::Succeeded
        }

        fn feed(
            &mut self,
            _step: StepId,
            _command: &Command,
            _input: &SecretInput,
            _sink: &mut dyn EventSink,
        ) -> Outcome {
            Outcome::Succeeded
        }
    }

    let mut runner = QueryFails;
    let mut events = Vec::new();
    let query = Command::new(Program::Pacman);
    let run = Command::new(Program::Pacman);

    let outcome = with_targets(StepId::new(1), &query, &run, &mut runner, &mut events);

    assert_eq!(
        outcome,
        Outcome::Failed {
            cause: Failure::Unknown,
            cleanup_failed: Vec::new()
        }
    );
}

#[test]
fn a_signal_end_reports_its_number() {
    use toolbox_pty::Exit;

    assert_eq!(
        super::super::super::report::ended(Exit::Signal(9_i32)),
        Outcome::Failed {
            cause: Failure::Signal { signal: 9_i32 },
            cleanup_failed: Vec::new()
        }
    );
}
