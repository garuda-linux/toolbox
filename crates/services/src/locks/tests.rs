use std::collections::BTreeMap;

use toolbox_core::features::ReadState;
use toolbox_plan::{Toggle, UserName};
use toolbox_system::LockProblem;
use toolbox_system::pkexec::{HelperError, Refusal};

use super::Locks;

fn user(name: &str) -> UserName {
    UserName::new(name).unwrap()
}

fn known() -> Locks {
    Locks::of_answer(Ok(BTreeMap::from([
        (user("ada"), false),
        (user("guest"), true),
    ])))
}

#[test]
fn an_answer_names_each_accounts_flag() {
    let locks = known();

    assert_eq!(locks.of(&user("guest")), Some(Toggle::On));
    assert_eq!(locks.of(&user("ada")), Some(Toggle::Off));
    assert_eq!(locks.of(&user("grace")), None, "not in the answer");
    assert!(locks.flags().is_some());
}

#[test]
fn a_refused_or_unknown_read_is_a_failed_guard_read() {
    let refused = Locks::of_answer(Err(LockProblem::Refused(Refusal::Denied)));
    let unknown = Locks::of_answer(Err(LockProblem::Failed(HelperError::TimedOut)));

    assert_eq!(refused.read_state(), ReadState::Failed);
    assert_eq!(unknown.read_state(), ReadState::Failed);
    for locks in [&Locks::NotAsked, &Locks::Loading, &known()] {
        assert_eq!(locks.read_state(), ReadState::Waiting, "{locks:?}");
    }
}

/// The guards wait for the flags, so a failed read gives them none.
#[test]
fn a_refused_or_failed_read_leaves_the_flags_unknown() {
    let refused = Locks::of_answer(Err(LockProblem::Refused(Refusal::Denied)));
    let failed = Locks::of_answer(Err(LockProblem::Failed(HelperError::TimedOut)));

    for locks in [&refused, &failed, &Locks::NotAsked, &Locks::Loading] {
        assert_eq!(locks.of(&user("ada")), None, "{locks:?}");
        assert_eq!(locks.flags(), None, "{locks:?}");
    }
}
