use std::collections::{BTreeSet, VecDeque};

use toolbox_plan::protocol::{Event, Failure, Mode, Outcome};
use toolbox_plan::{
    FullName, NewHome, NewUser, NewUserParts, Password, RootAction, StepId, UserName,
};

use super::{FakeRunner, finished, request, run};

const SECRET: &str = "correct horse battery staple";

const FAILED: Outcome = Outcome::Failed {
    cause: Failure::ExitCode { code: 9_i32 },
    cleanup_failed: Vec::new(),
};

fn user() -> UserName {
    UserName::new("ada").unwrap()
}

fn add_user(password: Option<Password>) -> Vec<RootAction> {
    vec![
        RootAction::CreateUser {
            user: NewUser::new(NewUserParts {
                name: user(),
                full_name: FullName::default(),
                shell: None,
                groups: BTreeSet::new(),
                home: NewHome::Standard,
                uid: None,
                system: false,
            })
            .unwrap(),
        },
        RootAction::SetPassword {
            user: user(),
            password,
        },
    ]
}

fn secret() -> Password {
    Password::new(SECRET.to_owned()).unwrap()
}

#[test]
fn a_new_user_is_created_and_then_given_the_password_on_stdin() {
    let mut runner = FakeRunner::default();
    let mut events = Vec::new();

    assert!(run(
        &request(Mode::Run, add_user(Some(secret()))),
        &mut runner,
        &mut events
    ));
    assert_eq!(runner.ran, ["useradd -m -- ada", "feed: chpasswd"]);
    assert_eq!(runner.fed, [format!("ada:{SECRET}\n").into_bytes()]);
    assert!(!format!("{events:?}").contains(SECRET));
    assert!(!runner.ran.join("\n").contains(SECRET));
}

#[test]
fn a_failed_useradd_keeps_the_password_from_being_set() {
    let mut runner = FakeRunner {
        outcomes: VecDeque::from([FAILED]),
        ..FakeRunner::default()
    };

    let mut events = Vec::new();

    assert!(!run(
        &request(Mode::Run, add_user(Some(secret()))),
        &mut runner,
        &mut events
    ));
    assert_eq!(finished(&events), [(StepId::new(1), FAILED)]);
    assert_eq!(runner.fed, Vec::<Vec<u8>>::new());
}

#[test]
fn a_password_step_without_its_secret_fails_and_runs_nothing() {
    let mut runner = FakeRunner::default();
    let mut events = Vec::new();
    let actions = vec![RootAction::SetPassword {
        user: user(),
        password: None,
    }];

    assert!(!run(&request(Mode::Run, actions), &mut runner, &mut events));
    assert!(runner.ran.is_empty() && runner.fed.is_empty());
    assert!(events.contains(&Event::Output {
        step: StepId::new(1),
        data: "the secret for chpasswd did not arrive\r\n".to_owned()
    }));
}

#[test]
fn a_dry_run_feeds_nothing_and_mentions_no_secret() {
    let mut runner = FakeRunner::default();
    let mut events = Vec::new();

    assert!(run(
        &request(Mode::DryRun, add_user(Some(secret()))),
        &mut runner,
        &mut events
    ));
    assert_eq!(
        finished(&events),
        [
            (StepId::new(1), Outcome::Skipped),
            (StepId::new(2), Outcome::Skipped)
        ]
    );
    assert!(runner.ran.is_empty() && runner.fed.is_empty());
    assert!(!format!("{events:?}").contains(SECRET));
}
