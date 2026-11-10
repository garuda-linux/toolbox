use toolbox_plan::protocol::Mode;

use super::Ending;

#[test]
fn every_step_that_changed_something_reads_as_applied() {
    assert_eq!(Ending::of(Mode::Run, 3, 0), Ending::Applied);
}

#[test]
fn steps_with_nothing_to_do_are_counted() {
    assert_eq!(Ending::of(Mode::Run, 2, 3), Ending::Partly { skipped: 3 });
}

#[test]
fn a_run_where_no_step_had_work_says_so() {
    assert_eq!(Ending::of(Mode::Run, 0, 4), Ending::Nothing);
}

#[test]
fn a_dry_run_is_a_test_whatever_the_steps_say() {
    assert_eq!(Ending::of(Mode::DryRun, 0, 4), Ending::Tested);
    assert_eq!(Ending::of(Mode::DryRun, 1, 0), Ending::Tested);
}

#[test]
fn a_plan_without_steps_reads_as_applied() {
    assert_eq!(Ending::of(Mode::Run, 0, 0), Ending::Applied);
}
