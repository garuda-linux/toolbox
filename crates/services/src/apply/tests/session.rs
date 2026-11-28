use toolbox_plan::protocol::{Event, Mode, Outcome, Request, encode};
use toolbox_plan::{RootAction, Step, StepId};

use super::{FakeLauncher, finished, root_plan, run, started};
use crate::apply::Report;

/// A plan with one root segment is one request of its steps, without
/// gates, and its events reach the reports.
#[tokio::test]
async fn one_root_segment_is_one_request_without_gates() {
    let dir = tempfile::tempdir().unwrap();
    let record = dir.path().join("stdin");
    let step = StepId::new(1);
    let events = [
        Event::Accepted,
        Event::Started { step },
        Event::Finished {
            step,
            outcome: Outcome::Succeeded,
        },
        Event::Done,
    ];

    let sent: String = events.iter().map(|event| encode(event).unwrap()).collect();
    let launcher = FakeLauncher {
        script: format!(
            "IFS= read -r line; printf '%s\\n' \"$line\" >> '{}'; printf '%s' '{sent}'",
            record.display()
        ),
        mode: Mode::Run,
    };

    let reports = run(root_plan(), launcher).await;

    let request = Request::new(
        Mode::Run,
        vec![Step {
            id: step,
            action: RootAction::RefreshKeyring,
        }],
    );

    assert_eq!(
        std::fs::read_to_string(&record).unwrap(),
        encode(&request).unwrap()
    );
    let mut expected = vec![started()];
    expected.extend(events.into_iter().map(Report::Step));
    expected.push(finished(true));
    assert_eq!(reports, expected);
}
