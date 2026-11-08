use std::sync::Arc;

use toolbox_cli::Exit;
use toolbox_services::apply::ApplyCommand;

use super::{Presses, Signal, Stopper, command, note};

#[test]
fn the_first_press_finishes_and_the_second_cancels() {
    assert!(matches!(command(1), Some(ApplyCommand::Finish)));
    assert!(matches!(command(2), Some(ApplyCommand::Cancel)));
    assert!(command(3).is_none());
}

#[test]
fn each_of_the_two_presses_says_what_happens_next() {
    assert_eq!(
        note(1),
        Some("stopping after this step, press Ctrl+C again to stop it now")
    );
    assert_eq!(note(2), Some("stopping now, the step's cleanup still runs"));
    assert_eq!(note(3), None);
}

/// A Ctrl+C of an earlier run, such as pkexec's before the sudo route,
/// does not count for this one.
#[test]
fn only_the_presses_of_this_run_count() {
    let presses = Arc::new(Presses::default());
    presses.press();

    let mut stopper = Stopper::new(Arc::clone(&presses));
    assert_eq!(stopper.take(), Vec::<u8>::new());

    presses.press();
    assert_eq!(stopper.take(), [1]);

    presses.press();
    presses.press();
    assert_eq!(stopper.take(), [2, 3]);
    assert_eq!(stopper.take(), Vec::<u8>::new());
}

/// SIGTERM and SIGHUP stop a change as Ctrl+C does, and the command ends
/// with 128 and the number of the first signal that came.
#[test]
fn a_signal_counts_as_a_press_and_names_the_exit_code() {
    let presses = Arc::new(Presses::default());
    let mut stopper = Stopper::new(Arc::clone(&presses));

    presses.signal(Signal::HangUp);
    presses.signal(Signal::Terminate);

    assert_eq!(stopper.take(), [1, 2]);
    assert_eq!(stopper.exit(), Exit::HungUp);
}

#[test]
fn each_signal_has_its_shell_code() {
    assert_eq!(Signal::Interrupt.exit(), Exit::Interrupted);
    assert_eq!(Signal::Terminate.exit(), Exit::Terminated);
    assert_eq!(Signal::HangUp.exit(), Exit::HungUp);
}

#[test]
fn a_press_is_ctrl_c() {
    let presses = Arc::new(Presses::default());
    let stopper = Stopper::new(Arc::clone(&presses));

    presses.press();

    assert_eq!(stopper.exit(), Exit::Interrupted);
}
