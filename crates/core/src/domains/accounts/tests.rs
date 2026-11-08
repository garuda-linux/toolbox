use std::collections::BTreeSet;

use super::Accounts;
use crate::testing::{NOBODY_ID, group, group_name, user, user_name};

fn accounts() -> Accounts {
    let mut nico = user("nico", 1000, 1000);
    nico.gecos = "Nico Jensch,Room 1,555".to_owned();

    Accounts {
        users: vec![
            user("ada", 1001, 100),
            nico,
            user("nobody", NOBODY_ID, NOBODY_ID),
            user("root", 0, 0),
            user("toor", 0, 0),
        ],
        groups: vec![
            group("nico", 1000, &[]),
            group("root", 0, &["root"]),
            group("users", 100, &[]),
            group("wheel", 998, &["nico", "ghost"]),
        ],
    }
}

#[test]
fn the_full_name_is_the_first_part_of_the_comment() {
    let accounts = accounts();

    assert_eq!(
        accounts.user(&user_name("nico")).unwrap().full_name(),
        "Nico Jensch"
    );
    assert_eq!(accounts.user(&user_name("ada")).unwrap().full_name(), "");
}

#[test]
fn regular_users_are_between_the_login_defs_bounds() {
    let accounts = accounts();
    let regular: Vec<&str> = accounts
        .users
        .iter()
        .filter(|user| user.is_regular())
        .map(|user| user.name.as_str())
        .collect();

    assert_eq!(regular, ["ada", "nico"]);
    assert!(user("edge", 60_000, 1).is_regular());
    assert!(!user("over", 60_001, 1).is_regular());
    assert!(!user("sys", 999, 1).is_regular());
}

#[test]
fn every_user_id_zero_is_root() {
    let accounts = accounts();
    let roots: Vec<&str> = accounts
        .users
        .iter()
        .filter(|user| user.is_root())
        .map(|user| user.name.as_str())
        .collect();

    assert_eq!(roots, ["root", "toor"]);
}

#[test]
fn membership_counts_listed_members_and_the_primary_group() {
    let accounts = accounts();

    assert!(accounts.is_member(&user_name("nico"), &group_name("wheel")));
    assert!(accounts.is_member(&user_name("nico"), &group_name("nico")));
    assert!(accounts.is_member(&user_name("ada"), &group_name("users")));
    assert!(!accounts.is_member(&user_name("ada"), &group_name("wheel")));
    assert!(!accounts.is_member(&user_name("ada"), &group_name("ghosts")));
    assert!(accounts.is_primary(&user_name("ada"), &group_name("users")));
    assert!(!accounts.is_primary(&user_name("nico"), &group_name("wheel")));

    assert_eq!(
        accounts.groups_of(&user_name("nico")),
        BTreeSet::from([group_name("nico"), group_name("wheel")])
    );
}

#[test]
fn members_include_primary_users_and_listed_names_without_an_account() {
    let accounts = accounts();

    assert_eq!(
        accounts.members_of(&group_name("wheel")),
        BTreeSet::from([user_name("ghost"), user_name("nico")])
    );
    assert_eq!(
        accounts.members_of(&group_name("users")),
        BTreeSet::from([user_name("ada")])
    );
    assert_eq!(
        accounts.members_of(&group_name("root")),
        BTreeSet::from([user_name("root"), user_name("toor")])
    );
}

#[test]
fn a_group_is_someones_primary_group_or_not() {
    let accounts = accounts();

    assert!(accounts.is_primary_of_someone(&group_name("users")));
    assert!(!accounts.is_primary_of_someone(&group_name("wheel")));
    assert!(!accounts.is_primary_of_someone(&group_name("ghosts")));
    assert!(accounts.group(&group_name("wheel")).unwrap().is_system());
    assert!(!accounts.group(&group_name("nico")).unwrap().is_system());
}
