use super::super::add_group;
use super::super::listing::{GroupOrder, UserOrder, arrange_groups, arrange_users};
use super::super::rows::{group_choices, group_rows, user_rows};
use super::super::view::group_names;
use crate::domains::{Accounts, CurrentUser};
use crate::test_support::{group, group_name, user, user_name};
use crate::{ChangeSet, Snapshot};

/// Four accounts whose fields each sort another way.
fn snapshot() -> Snapshot {
    let mut ada = user("ada", 1002, 1002);
    ada.gecos = "Zoe Adams,Room 4".to_owned();
    ada.home = "/srv/people/ada".to_owned();
    ada.shell = "/usr/bin/zsh".to_owned();

    let mut bob = user("bob", 1001, 1001);
    bob.gecos = "anna Bobson".to_owned();
    bob.shell = String::new();

    let mut cy = user("cy", 1000, 1000);
    cy.gecos = String::new();
    cy.shell = "/usr/bin/Fish".to_owned();

    let mut snapshot = Snapshot::default();

    snapshot.insert(Accounts {
        users: vec![ada, bob, cy, user("http", 33, 33)],
        groups: vec![
            group("ada", 1002, &[]),
            group("bob", 1001, &[]),
            group("cy", 1000, &[]),
            group("http", 33, &[]),
            group("makers", 1500, &["ada", "bob", "cy"]),
            group("wheel", 998, &["cy", "bob"]),
        ],
    });
    snapshot.insert(CurrentUser(user_name("cy")));
    snapshot
}

fn users(search: &str, order: UserOrder) -> Vec<String> {
    let snapshot = snapshot();
    let accounts = snapshot.part::<Accounts>().unwrap();
    let mut rows = user_rows(accounts, &snapshot, &ChangeSet::default(), false);

    arrange_users(&mut rows, search, order);
    rows.iter().map(|row| row.user.name.to_string()).collect()
}

fn groups(search: &str, order: GroupOrder) -> Vec<String> {
    let snapshot = snapshot();
    let accounts = snapshot.part::<Accounts>().unwrap();
    let mut changes = ChangeSet::default();

    add_group(&group_name("crafts"), &mut changes, &snapshot).unwrap();

    let mut rows = group_rows(accounts, &changes);

    arrange_groups(&mut rows, search, order);
    rows.iter().map(|row| row.name.to_string()).collect()
}

#[test]
fn users_are_found_by_every_field() {
    for (search, found) in [
        ("", vec!["ada", "bob", "cy", "http"]),
        ("  ", vec!["ada", "bob", "cy", "http"]),
        ("AD", vec!["ada"]),
        ("1001", vec!["bob"]),
        ("100", vec!["ada", "bob", "cy"]),
        ("bobson", vec!["bob"]),
        ("room", vec![]),
        ("/srv", vec!["ada"]),
        ("fish", vec!["cy"]),
        ("wheel", vec!["bob", "cy"]),
        (" Makers ", vec!["ada", "bob", "cy"]),
        ("nothing", vec![]),
    ] {
        assert_eq!(users(search, UserOrder::Name), found, "{search:?}");
    }
}

#[test]
fn users_sort_by_each_field_with_the_name_as_tie_break() {
    for (order, sorted) in [
        (UserOrder::Name, ["ada", "bob", "cy", "http"]),
        (UserOrder::Id, ["http", "cy", "bob", "ada"]),
        (UserOrder::FullName, ["bob", "ada", "cy", "http"]),
        (UserOrder::Home, ["bob", "cy", "http", "ada"]),
        (UserOrder::Shell, ["http", "cy", "ada", "bob"]),
    ] {
        assert_eq!(users("", order), sorted, "{order:?}");
    }

    assert_eq!(users("100", UserOrder::Id), ["cy", "bob", "ada"]);
}

#[test]
fn groups_are_found_by_name_id_and_member() {
    for (search, found) in [
        ("WHE", vec!["wheel"]),
        ("998", vec!["wheel"]),
        ("ada", vec!["ada", "makers"]),
        ("craft", vec!["crafts"]),
        ("nothing", vec![]),
    ] {
        assert_eq!(groups(search, GroupOrder::Name), found, "{search:?}");
    }
}

#[test]
fn groups_sort_by_name_id_or_size_and_a_queued_one_has_no_id_yet() {
    assert_eq!(
        groups("", GroupOrder::Name),
        ["ada", "bob", "crafts", "cy", "http", "makers", "wheel"]
    );
    assert_eq!(
        groups("", GroupOrder::Id),
        ["http", "wheel", "cy", "bob", "ada", "makers", "crafts"]
    );
    assert_eq!(
        groups("", GroupOrder::Members),
        ["makers", "wheel", "ada", "bob", "cy", "http", "crafts"]
    );
}

#[test]
fn the_group_dialog_knows_id_and_size_of_every_group() {
    let snapshot = snapshot();
    let accounts = snapshot.part::<Accounts>().unwrap();
    let mut changes = ChangeSet::default();

    add_group(&group_name("crafts"), &mut changes, &snapshot).unwrap();

    let choices = group_choices(&user_name("ada"), accounts, &changes);
    let shown: Vec<(&str, Option<u32>, usize)> = choices
        .iter()
        .map(|choice| (choice.group.as_str(), choice.gid, choice.members))
        .collect();

    assert_eq!(
        shown,
        [
            ("ada", Some(1002), 1),
            ("bob", Some(1001), 1),
            ("crafts", None, 0),
            ("cy", Some(1000), 1),
            ("http", Some(33), 1),
            ("makers", Some(1500), 3),
            ("wheel", Some(998), 2),
        ]
    );
}

/// The add-user dialog finds a group by its name alone, and offers no
/// group that is only queued: it does not exist yet.
#[test]
fn the_group_picker_finds_names_only_and_leaves_out_queued_groups() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    add_group(&group_name("crafts"), &mut changes, &snapshot).unwrap();

    let names = |search: &str| -> Vec<String> {
        group_names(&snapshot, &changes, search)
            .iter()
            .map(ToString::to_string)
            .collect()
    };

    assert_eq!(names(" MAK "), ["makers"]);
    assert_eq!(names("998"), Vec::<String>::new(), "no group by its id");
    assert_eq!(names("bob"), ["bob"], "no group by a member");
    assert_eq!(names("cra"), Vec::<String>::new());
    assert_eq!(names("").len(), 6);
}
