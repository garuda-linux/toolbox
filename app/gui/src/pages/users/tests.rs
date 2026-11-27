mod add_user;
mod dialogs;
mod edit;
mod groups;
mod membership;
mod password;
mod rows;

use std::collections::BTreeMap;
use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::domains::{Accounts, AvailableShells, CurrentUser, Group, User};
use toolbox_core::features::users::Refusal;
use toolbox_core::features::users::run::{PageRun, RunKind};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui_kit::icons;
use toolbox_gui_kit::widgets::ButtonKind;
use toolbox_plan::{GroupName, HomeDirectory, ShellPath, UserName};
use toolbox_services::locks::Locks;

use super::UsersPage;
use crate::command::{AppCommand, Command, UserEdit, UsersCommand};
use crate::page::PageId;
use crate::page_service::Services;
use crate::refused::Refused;
use crate::testing::{PageFixture, PageState, assert_header, button_kind, paints_icon, rect_of};
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
        gecos: toolbox_text::plain(gecos),
        home: Ok(HomeDirectory::new(format!("/home/{text}")).unwrap()),
        shell: Ok(ShellPath::new("/usr/bin/bash").unwrap()),
    }
}

fn group(text: &str, gid: u32, members: &[&str]) -> Group {
    Group {
        name: group_name(text),
        gid,
        members: members.iter().map(|member| name(member)).collect(),
    }
}

/// `grace` (logged in, admin), `ada` and the service `http`, and the group
/// `video` beside their own ones.
pub fn system() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(Accounts {
        users: vec![
            user("ada", 1001, "Ada Lovelace"),
            user("http", 33, ""),
            user("grace", 1000, "Grace Hopper"),
        ],
        groups: vec![
            group("ada", 1001, &[]),
            group("http", 33, &[]),
            group("grace", 1000, &[]),
            group("video", 1500, &["ada"]),
            group("wheel", 998, &["grace"]),
        ],
    });
    snapshot.insert(CurrentUser(name("grace")));
    snapshot.insert(AvailableShells(vec![
        ShellPath::new("/usr/bin/bash").unwrap(),
        ShellPath::new("/usr/bin/fish").unwrap(),
    ]));
    snapshot
}

fn unlocked() -> Users {
    let flags = BTreeMap::from([(name("ada"), false), (name("grace"), false)]);
    Users::showing(Locks::Known(flags), PageRun::None)
}

pub fn page(users: Users) -> Shown {
    page_on(system(), users)
}

pub fn page_on(snapshot: Snapshot, users: Users) -> Shown {
    page_with_queue(snapshot, users, ChangeSet::default())
}

pub fn page_with_queue(snapshot: Snapshot, users: Users, changes: ChangeSet) -> Shown {
    let mut harness = PageFixture {
        snapshot: Arc::new(snapshot),
        changes,
        services: Services {
            users,
            ..Services::default()
        },
        ..PageFixture::default()
    }
    .harness(UsersPage::default());

    harness.run();
    harness
}

/// The page with the dialog `entry` of the row menu of `user` open.
pub fn from_menu(user: &str, entry: &str) -> Shown {
    let mut harness = page(unlocked());
    harness
        .get_by_label(&format!("More actions for {user}"))
        .click();
    harness.run();
    harness.get_by_label(entry).click();
    harness.run();
    harness
}

pub fn sent(harness: &Shown) -> Vec<Command> {
    harness.state().fixture.sent()
}

#[test]
fn people_show_and_the_service_waits_behind_the_filter() {
    let mut harness = page(unlocked());

    assert!(harness.query_by_label("Showing 2 of 3 accounts").is_some());
    assert!(harness.query_by_label("http").is_none());

    harness
        .get_by_role_and_label(Role::Button, "Regular Users Only")
        .click();
    harness.run();

    assert!(harness.query_by_label("http").is_some());
}

#[test]
fn the_menu_queues_a_lock() {
    let mut harness = page(unlocked());

    harness.get_by_label("More actions for ada").click();
    harness.run();
    harness.get_by_label("Queue Lock").click();
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
    let mut harness = page(users);

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
    let mut harness = page(Users::showing(Locks::Unknown, run));

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
    let mut users = unlocked();
    users.set_refusal(Some(Refused {
        refusal: Refusal::LastAdmin,
        subject: "grace".to_owned(),
    }));

    let mut harness = page(users);

    assert!(
        harness
            .query_by_label_contains("grace is the last member of the groups wheel and sudo")
            .is_some()
    );

    harness.get_by_label("Dismiss").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(AppCommand::DismissRefusal(PageId::Users))]
    );
}

#[test]
fn the_groups_tab_removes_a_group_after_asking() {
    let mut harness = page(unlocked());

    harness
        .get_by_role_and_label(Role::RadioButton, "Groups")
        .click();
    harness.run();

    assert!(harness.query_by_label_contains("Group ID 1500").is_some());

    harness.get_by_label("Remove video…").click();
    harness.run();
    harness.get_by_label("Queue Removal").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(UserEdit::RemoveGroup(group_name("video")))]
    );
}

#[test]
fn the_header_stays_put_and_each_tab_sorts_in_the_toolbar() {
    let mut harness = page(unlocked());
    let texts = (
        "Users and Groups",
        "Add users, set passwords, lock accounts and change groups.",
    );

    let tab = rect_of(&harness, Role::RadioButton, "Users");
    let refresh = rect_of(&harness, Role::Button, "Read Again");
    let search = rect_of(&harness, Role::TextInput, "Search users");
    assert_header(&harness, texts, Some(tab), Some(search));
    let sort = rect_of(&harness, Role::ComboBox, "Sort users");
    assert!(
        sort.right() <= refresh.left(),
        "sort {sort:?} is not before refresh {refresh:?}"
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
        groups.right() <= refresh.left(),
        "sort {groups:?} is not before refresh {refresh:?}"
    );
}

#[test]
fn removing_a_group_is_offered_and_confirmed_softly() {
    let mut harness = page(unlocked());
    harness
        .get_by_role_and_label(Role::RadioButton, "Groups")
        .click();
    harness.run();

    assert_eq!(button_kind(&harness, "Remove video…"), ButtonKind::Soft);
    harness.get_by_label("Remove video…").click();
    harness.run();

    // KDE has no red button, so a Danger answer draws as Soft (buttons.rs)
    // and only its delete icon marks it.
    assert_eq!(button_kind(&harness, "Queue Removal"), ButtonKind::Soft);
    let answer = rect_of(&harness, Role::Button, "Queue Removal");
    assert!(paints_icon(&harness, answer, icons::EDIT_DELETE));
}

#[test]
fn every_row_offers_edit_and_its_menu_at_its_right() {
    let harness = page(unlocked());

    assert!(harness.query_by_label("Set Password…").is_none());
    for user in ["ada", "grace"] {
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
        services: Services {
            users: unlocked(),
            ..Services::default()
        },
        busy: true,
        ..PageFixture::default()
    }
    .harness(UsersPage::default());

    harness.run();
    harness.get_by_label("More actions for ada").click();
    harness.run();
    assert!(
        harness
            .get_by_role_and_label(Role::MenuItem, "Set Password…")
            .accesskit_node()
            .is_disabled()
    );
}

#[test]
fn the_edit_dialog_shows_the_shell_by_name_and_tells_its_path() {
    let mut harness = page(unlocked());
    harness
        .get_by_role_and_label(Role::Button, "Edit ada…")
        .click();
    harness.run();

    let shell = harness.get_by_role_and_label(Role::ComboBox, "Login shell");
    let node = shell.accesskit_node();
    assert_eq!(node.value().as_deref(), Some("bash"));
    assert_eq!(node.description().as_deref(), Some("/usr/bin/bash"));
}
