use toolbox_core::features::console::HelperFailure;
use toolbox_plan::protocol::Mode;
use toolbox_plan::{Password, Plan, PlannedAction, RootAction, UserName};

use super::{FakeLauncher, recording_script, run};
use crate::apply::{Problem, Report};

const SECRET: &str = "correct horse battery staple";

fn password_plan(password: Option<Password>) -> Plan {
    Plan::new([
        PlannedAction::Root(RootAction::RefreshKeyring),
        PlannedAction::Root(RootAction::SetPassword {
            user: UserName::new("ada").unwrap(),
            password,
        }),
    ])
}

fn secret() -> Password {
    Password::new(SECRET.to_owned()).unwrap()
}

#[tokio::test]
async fn the_password_follows_the_request_on_its_own_line() {
    let dir = tempfile::tempdir().unwrap();
    let record = dir.path().join("stdin");
    let launcher = FakeLauncher {
        script: recording_script(&record, 2),
        mode: Mode::Run,
    };

    let reports = run(password_plan(Some(secret())), launcher).await;
    let received = std::fs::read_to_string(&record).unwrap();
    let mut lines = received.lines();
    let request = lines.next().unwrap();

    assert!(request.contains("set-password") && !request.contains(SECRET));
    assert_eq!(lines.next(), Some(format!("secret 2 {SECRET}").as_str()));
    assert!(!format!("{reports:?}").contains(SECRET));
}

#[tokio::test]
async fn a_dry_run_sends_no_password_at_all() {
    let dir = tempfile::tempdir().unwrap();
    let record = dir.path().join("stdin");
    let launcher = FakeLauncher {
        script: recording_script(&record, 1),
        mode: Mode::DryRun,
    };

    run(password_plan(Some(secret())), launcher).await;
    let received = std::fs::read_to_string(&record).unwrap();

    assert_eq!(received.lines().count(), 1);
    assert!(!received.contains(SECRET));
}

#[tokio::test]
async fn a_password_step_without_a_password_never_starts_the_helper() {
    let dir = tempfile::tempdir().unwrap();
    let record = dir.path().join("stdin");
    let launcher = FakeLauncher {
        script: recording_script(&record, 1),
        mode: Mode::Run,
    };

    let reports = run(password_plan(None), launcher).await;

    assert!(!record.exists());
    assert!(reports.iter().any(|report| matches!(
        report,
        Report::Problem(Problem::HelperFailed { kind: HelperFailure::MissingSecret, detail })
            if detail == "2"
    )));
}
