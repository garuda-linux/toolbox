use toolbox_plan::StepId;
use toolbox_plan::protocol::Event;

use crate::apply::{ApplyHandle, Report};
use crate::waking::Wake;

const KIB: usize = 1024;

fn output(bytes: usize) -> Report {
    Report::Step(Event::Output {
        step: StepId::new(1),
        data: "x".repeat(bytes),
    })
}

#[test]
fn a_mebibyte_of_output_comes_in_parts_of_the_budget() {
    let (reports, handle) = ApplyHandle::feeding(Wake::none());
    for _ in 0..256_u32 {
        reports.send(output(4 * KIB));
    }

    let (first, more) = handle.reports_within(128 * KIB);

    assert_eq!(first.len(), 32);
    assert!(more, "the rest waits for the next frame");
}

#[test]
fn reports_without_output_cost_nothing() {
    let (reports, handle) = ApplyHandle::feeding(Wake::none());
    for _ in 0..1000_u32 {
        reports.send(Report::CancelWaits);
    }

    let (taken, more) = handle.reports_within(128 * KIB);

    assert_eq!(taken.len(), 1000);
    assert!(!more);
}

#[test]
fn one_report_larger_than_the_budget_still_goes() {
    let (reports, handle) = ApplyHandle::feeding(Wake::none());
    reports.send(output(512 * KIB));
    reports.send(Report::PlanFinished { succeeded: true });

    let (first, more) = handle.reports_within(128 * KIB);
    assert_eq!(first, [output(512 * KIB)]);
    assert!(more, "the next report waits for the next frame");

    let (rest, after) = handle.reports_within(128 * KIB);
    assert_eq!(rest, [Report::PlanFinished { succeeded: true }]);
    assert!(!after, "nothing waits after the last report");
}
