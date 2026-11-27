use std::collections::BTreeSet;

use toolbox_plan::procedure::Invocation;
use toolbox_plan::{FullName, Password, PlannedAction, RootAction, Segment, ShellPath};

use super::super::Refusal;
use super::super::now::{check_new_name, new_user, plan_add_user, plan_set_password};
use super::{groups, snapshot};
use crate::Snapshot;
use crate::test_support::user_name;

const SECRET: &str = "correct horse battery staple";

fn password() -> Password {
    Password::new(SECRET.to_owned()).unwrap()
}

fn fish() -> ShellPath {
    ShellPath::new("/usr/bin/fish").unwrap()
}

#[test]
fn a_new_user_needs_a_free_name_a_listed_shell_and_known_groups() {
    let snapshot = snapshot();
    let name = FullName::new("Grace Hopper").unwrap();

    for (user, shell, in_groups, refusal) in [
        ("grace", Some(fish()), groups(&[]), Refusal::UserExists),
        ("wheel", Some(fish()), groups(&[]), Refusal::GroupExists),
        (
            "hopper",
            Some(ShellPath::new("/usr/bin/false").unwrap()),
            groups(&[]),
            Refusal::ShellNotListed,
        ),
        (
            "hopper",
            Some(fish()),
            groups(&["ghosts"]),
            Refusal::UnknownGroup,
        ),
    ] {
        assert_eq!(
            new_user(user_name(user), name.clone(), shell, in_groups, &snapshot),
            Err(refusal),
            "{user}"
        );
    }

    let hopper = new_user(
        user_name("hopper"),
        name,
        None,
        groups(&["audio", "wheel"]),
        &snapshot,
    )
    .unwrap();

    assert_eq!(hopper.parts().name, user_name("hopper"));
    assert_eq!(hopper.parts().groups, groups(&["audio", "wheel"]));
    assert_eq!(check_new_name(&user_name("hopper"), &snapshot), Ok(()));
    assert_eq!(
        check_new_name(&user_name("hopper"), &Snapshot::default()),
        Err(Refusal::Unavailable)
    );
}

#[test]
fn adding_a_user_is_one_root_segment_of_useradd_then_chpasswd() {
    let snapshot = snapshot();
    let hopper = new_user(
        user_name("hopper"),
        FullName::default(),
        Some(fish()),
        BTreeSet::new(),
        &snapshot,
    )
    .unwrap();

    let plan = plan_add_user(hopper, password());
    let segments = plan.into_segments();
    let [Segment::Root(steps)] = segments.as_slice() else {
        panic!("one root segment expected");
    };

    let shown: Vec<String> = steps
        .iter()
        .flat_map(|step| step.action.procedure())
        .map(|invocation| invocation.to_string())
        .collect();

    assert_eq!(
        shown,
        [
            "check that /usr/bin/fish is listed in /etc/shells",
            "useradd -m -s /usr/bin/fish -- hopper",
            "chpasswd  # the secret arrives on its standard input",
        ]
    );
    assert!(matches!(
        steps.last().map(|step| &step.action),
        Some(RootAction::SetPassword {
            user: _,
            password: Some(_)
        })
    ));
}

#[test]
fn a_password_plan_never_shows_its_password() {
    let snapshot = snapshot();
    let plan = plan_set_password(&user_name("grace"), password(), &snapshot).unwrap();
    let procedure: Vec<_> = plan
        .steps()
        .iter()
        .flat_map(|step| match &step.action {
            PlannedAction::Root(action) => action.procedure(),
            PlannedAction::User(action) => action.procedure(),
        })
        .collect();

    let shown: Vec<String> = procedure.iter().map(ToString::to_string).collect();
    let texts = [
        format!("{plan:?}"),
        format!("{:?}", plan.clone().into_segments()),
        format!("{procedure:?}"),
        shown.join("\n"),
    ];

    for text in texts {
        assert!(!text.contains(SECRET), "{text}");
    }

    assert!(matches!(
        plan.steps().first().map(|step| &step.action),
        Some(PlannedAction::Root(RootAction::SetPassword {
            user: _,
            password: Some(_)
        }))
    ));
    assert!(plan.steps().iter().any(|step| match &step.action {
        PlannedAction::Root(action) => action.procedure().iter().any(|invocation| matches!(
            invocation,
            Invocation::Feed {
                command: _,
                input: Some(_)
            }
        )),
        PlannedAction::User(_) => false,
    }));
}

#[test]
fn root_and_unknown_users_get_no_password_here() {
    let snapshot = snapshot();

    assert_eq!(
        plan_set_password(&user_name("root"), password(), &snapshot),
        Err(Refusal::Root)
    );
    assert_eq!(
        plan_set_password(&user_name("ghost"), password(), &snapshot),
        Err(Refusal::UnknownUser)
    );
    assert_eq!(
        plan_set_password(&user_name("grace"), password(), &Snapshot::default()),
        Err(Refusal::Unavailable)
    );
}
