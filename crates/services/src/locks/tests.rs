use std::collections::BTreeMap;

use toolbox_plan::UserName;
use toolbox_system::LockProblem;
use toolbox_system::pkexec::Refusal;

use super::Locks;

fn user(name: &str) -> UserName {
    UserName::new(name).unwrap()
}

fn known() -> Locks {
    Locks::of_answer(Ok(BTreeMap::from([
        (user("nico"), false),
        (user("guest"), true),
    ])))
}

#[test]
fn an_answer_names_each_accounts_flag() {
    let locks = known();

    assert_eq!(locks.of(&user("guest")), Some(true));
    assert_eq!(locks.of(&user("nico")), Some(false));
    assert_eq!(locks.of(&user("ada")), None, "not in the answer");
    assert_eq!(locks.code(), "known");
    assert!(locks.flags().is_some());
}

/// The guards wait for the flags, so a failed read gives them none.
#[test]
fn a_refused_or_failed_read_leaves_the_flags_unknown() {
    let refused = Locks::of_answer(Err(LockProblem::Refused(Refusal::Denied)));
    let failed = Locks::of_answer(Err(LockProblem::Failed("no answer".to_owned())));

    for locks in [&refused, &failed, &Locks::NotAsked, &Locks::Loading] {
        assert_eq!(locks.of(&user("nico")), None, "{locks:?}");
        assert_eq!(locks.flags(), None, "{locks:?}");
    }

    assert_eq!(refused.code(), "denied");
    assert_eq!(failed.code(), "unknown");
    assert_eq!(Locks::NotAsked.code(), "loading");
}
