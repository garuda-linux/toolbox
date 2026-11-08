mod dialogs;

use std::collections::BTreeMap;
use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::Snapshot;
use toolbox_core::domains::{Accounts, AvailableShells, CurrentUser, Group, User};
use toolbox_core::features::users::run::{PageRun, RunKind};
use toolbox_gui_kit::widgets::ButtonKind;
use toolbox_plan::{GroupName, ShellPath, UserName};
use toolbox_system::Locks;

use super::UsersPage;
use crate::command::{Command, UserEdit, UsersCommand};
use crate::pages::{Refusals, UsersRefusal};
use crate::testing::{PageFixture, PageState, assert_header, button_kind, rect_of};
use crate::users::Users;

type Shown = Harness<'static, PageState<UsersPage>>;

fn name(text: &str) -> UserName {
    UserName::new(text).unwrap()
}

fn group_name(text: &str) -> GroupName {
    GroupName::new(text).unwrap()
}

fn user(text: &str, uid: u32, gecos: &str) -> User {
    User {
        name: name(text),
        uid,
        gid: uid,
        gecos: gecos.to_owned(),
        home: format!("/home/{text}"),
        shell: "/usr/bin/bash".to_owned(),
    }
}

fn group(text: &str, gid: u32, members: &[&str]) -> Group {
    Group {
        name: group_name(text),
        gid,
        members: members.iter().map(|member| name(member)).collect(),
    }
}

/// `nico` (logged in, admin), `ada` and the service `http`, and the group
/// `video` beside their own ones.
pub fn system() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(Accounts {
        users: vec![
            user("ada", 1001, "Ada Lovelace"),
            user("http", 33, ""),
            user("nico", 1000, "Nico"),
        ],
        groups: vec![
            group("ada", 1001, &[]),
            group("http", 33, &[]),
            group("nico", 1000, &[]),
            group("video", 1500, &["ada"]),
            group("wheel", 998, &["nico"]),
        ],
    });
    snapshot.insert(CurrentUser(name("nico")));
    snapshot.insert(AvailableShells(vec![
        ShellPath::new("/usr/bin/bash").unwrap(),
        ShellPath::new("/usr/bin/fish").unwrap(),
    ]));
    snapshot
}

fn unlocked() -> Users {
    let flags = BTreeMap::from([(name("ada"), false), (name("nico"), false)]);
    Users::showing(Locks::Known(flags), PageRun::None)
}

pub fn page(users: Users, refusals: Refusals) -> Shown {
    page_on(system(), users, refusals)
}

pub fn page_on(snapshot: Snapshot, users: Users, refusals: Refusals) -> Shown {
    let mut harness = PageFixture {
        snapshot: Arc::new(snapshot),
        users,
        refusals,
        ..PageFixture::default()
    }
    .harness(UsersPage::default());

    harness.run();
    harness
}

pub fn sent(harness: &Shown) -> Vec<Command> {
    harness.state().fixture.sent()
}

#[test]
fn people_show_and_the_service_waits_behind_the_filter() {
    let mut harness = page(unlocked(), Refusals::default());

    assert!(harness.query_by_label("Showing 2 of 3 accounts").is_some());
    assert!(harness.query_by_label("http").is_none());

    harness
        .get_by_role_and_label(Role::Button, "Regular users only")
        .click();
    harness.run();

    assert!(harness.query_by_label("http").is_some());
}

#[test]
fn the_menu_queues_a_lock() {
    let mut harness = page(unlocked(), Refusals::default());

    harness.get_by_label("More actions for ada").click();
    harness.run();
    harness.get_by_label("Queue lock").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(UserEdit::SetLocked {
            user: name("ada"),
            locked: true,
        })]
    );
}

#[test]
fn without_the_lock_state_the_page_says_so_and_locks_nobody() {
    let users = Users::showing(Locks::Unknown, PageRun::None);
    let mut harness = page(users, Refusals::default());

    assert!(
        harness
            .query_by_label_contains("Which accounts are locked is not known")
            .is_some()
    );

    harness.get_by_label("More actions for ada").click();
    harness.run();

    assert!(harness.query_by_label("Lock: state not known").is_some());
}

#[test]
fn a_failed_run_says_so_and_offers_the_console() {
    let run = PageRun::Failed(RunKind::Password, name("ada"));
    let mut harness = page(Users::showing(Locks::Unknown, run), Refusals::default());

    assert!(
        harness
            .query_by_label_contains("The password of ada could not be set")
            .is_some()
    );

    harness.get_by_label("Show Console").click();
    harness.run();

    assert_eq!(sent(&harness), [Command::from(UsersCommand::ShowConsole)]);
}

#[test]
fn a_refused_click_names_the_account_until_dismissed() {
    let refusals = Refusals {
        users: Some(UsersRefusal {
            code: "last-admin",
            subject: "nico".to_owned(),
        }),
        ..Refusals::default()
    };

    let mut harness = page(unlocked(), refusals);

    assert!(
        harness
            .query_by_label_contains("nico is the last member of the groups wheel and sudo")
            .is_some()
    );

    harness.get_by_label("Dismiss").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(UsersCommand::DismissRefusal)]
    );
}

#[test]
fn the_groups_tab_removes_a_group_after_asking() {
    let mut harness = page(unlocked(), Refusals::default());

    harness
        .get_by_role_and_label(Role::RadioButton, "Groups")
        .click();
    harness.run();

    assert!(harness.query_by_label_contains("Group ID 1500").is_some());

    harness.get_by_label("Remove video…").click();
    harness.run();
    harness.get_by_label("Queue removal").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(UserEdit::RemoveGroup(group_name("video")))]
    );
}

#[test]
fn the_header_stays_put_and_each_tab_sorts_in_the_toolbar() {
    let mut harness = page(unlocked(), Refusals::default());
    let texts = (
        "Users and Groups",
        "Add users, set passwords, lock accounts and change groups.",
    );

    let tab = rect_of(&harness, Role::RadioButton, "Users");
    let refresh = rect_of(&harness, Role::Button, "Read again");
    let search = rect_of(&harness, Role::TextInput, "Search users");
    assert_header(&harness, texts, Some(tab), Some(search));
    let sort = rect_of(&harness, Role::ComboBox, "Sort users");
    assert!(
        (sort.right() - refresh.right()).abs() < 0.5,
        "sort {sort:?}, refresh {refresh:?}"
    );
    assert!(
        (sort.center().y - search.center().y).abs() < 0.5,
        "not one toolbar row"
    );

    harness
        .get_by_role_and_label(Role::RadioButton, "Groups")
        .click();
    harness.run();

    assert_eq!(rect_of(&harness, Role::RadioButton, "Users"), tab);
    let groups = rect_of(&harness, Role::ComboBox, "Sort groups");
    assert!(
        (groups.right() - refresh.right()).abs() < 0.5,
        "sort {groups:?}, refresh {refresh:?}"
    );
}

#[test]
fn removing_a_group_is_offered_softly_and_confirmed_as_danger() {
    let mut harness = page(unlocked(), Refusals::default());
    harness
        .get_by_role_and_label(Role::RadioButton, "Groups")
        .click();
    harness.run();

    assert_eq!(button_kind(&harness, "Remove video…"), ButtonKind::Soft);
    harness.get_by_label("Remove video…").click();
    harness.run();
    assert_eq!(button_kind(&harness, "Queue removal"), ButtonKind::Danger);
}

#[test]
fn every_row_offers_edit_and_its_menu_at_its_right() {
    let harness = page(unlocked(), Refusals::default());

    assert!(harness.query_by_label("Set password…").is_none());
    for user in ["ada", "nico"] {
        let edit_name = format!("Edit {user}…");
        let more_name = format!("More actions for {user}");
        let edit = harness.get_by_role_and_label(Role::Button, &edit_name);
        let more = harness.get_by_role_and_label(Role::Button, &more_name);
        let row = harness.get_by_label(user).rect();
        assert!(
            edit.rect().left() > row.right(),
            "{user}: Edit sits at the right of the row's words"
        );
        assert!(more.rect().left() > edit.rect().right());
    }
}

#[test]
fn the_menu_rests_the_password_while_a_plan_runs() {
    let mut harness = PageFixture {
        snapshot: Arc::new(system()),
        users: unlocked(),
        busy: true,
        ..PageFixture::default()
    }
    .harness(UsersPage::default());

    harness.run();
    harness.get_by_label("More actions for ada").click();
    harness.run();
    assert!(
        harness
            .get_by_role_and_label(Role::MenuItem, "Set password…")
            .accesskit_node()
            .is_disabled()
    );
}

#[test]
fn the_edit_dialog_shows_the_shell_by_name_and_tells_its_path() {
    let mut harness = page(unlocked(), Refusals::default());
    harness
        .get_by_role_and_label(Role::Button, "Edit ada…")
        .click();
    harness.run();

    let shell = harness.get_by_role_and_label(Role::ComboBox, "Login shell");
    let node = shell.accesskit_node();
    assert_eq!(node.value().as_deref(), Some("bash"));
    assert_eq!(node.description().as_deref(), Some("/usr/bin/bash"));
}
