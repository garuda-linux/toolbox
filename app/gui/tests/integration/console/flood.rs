//! A program that prints faster than a frame can draw: its output reaches
//! the Console over several frames, and the window keeps drawing.

use egui_kittest::kittest::Queryable as _;
use toolbox_gui::apply::{ApplyHandle, Report};
use toolbox_gui::waking::Wake;
use toolbox_gui::{App, Setup};
use toolbox_plan::StepId;
use toolbox_plan::protocol::{Event, Outcome};

use crate::common::{HEIGHT, window};

const KIB: usize = 1024;

/// The note of a run that ended while the Console was closed.
const APPLIED: &str = "Your changes were applied.";

/// A mebibyte of output in lines of 4 KiB, then the plan's end.
fn flood() -> Vec<Report> {
    let step = StepId::new(1);
    let line = format!("{}\r\n", "x".repeat(4 * KIB - 2));
    let output = std::iter::repeat_n(Report::Step(Event::Output { step, data: line }), 256);

    [
        Report::PlanStarted {
            steps: vec![(step, vec!["pacman -S java-runtime".to_owned()])],
        },
        Report::Step(Event::Started { step }),
    ]
    .into_iter()
    .chain(output)
    .chain([
        Report::Step(Event::Finished {
            step,
            outcome: Outcome::Succeeded,
        }),
        Report::PlanFinished { succeeded: true },
    ])
    .collect()
}

#[test]
fn a_mebibyte_of_output_spreads_over_frames_and_all_of_it_arrives() {
    let (reports, apply) = ApplyHandle::feeding(Wake::none());
    let mut harness = window(HEIGHT).with_max_steps(64).build_eframe(move |cc| {
        App::new(
            cc,
            Setup {
                apply: Some(apply),
                ..Setup::default()
            },
        )
    });

    harness.run();

    for report in flood() {
        reports.send(report);
    }

    harness.step();
    assert!(
        harness.query_by_label(APPLIED).is_none(),
        "one frame took the whole flood"
    );

    // Nothing but the app's own repaints wakes these frames.
    harness.run();
    assert!(harness.query_by_label(APPLIED).is_some());
}
