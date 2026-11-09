//! The Users and groups page with the real queue and a fake helper: the
//! lock flags once the page opens, a queued lock and a queued group, and
//! the snapshots.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use tokio::runtime::Runtime;
use toolbox_core::domains::{AccountLocks, Accounts, AvailableShells, CurrentUser, Group, User};
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_gui::queue::Queue;
use toolbox_gui::users::UsersSetup;
use toolbox_gui::{App, Setup, snapshot};
use toolbox_plan::{GroupName, ShellPath, UserName};
use toolbox_system::{LockProblem, LockSource};

use crate::common::{HEIGHT, type_into, window};
use crate::system::{first_read, settle, step_until};

/// Answers that `ada` is locked, and counts the reads.
#[derive(Default)]
struct Helper(AtomicUsize);

/// The app on Users and groups. The runtime reads the system, so it lives
/// as long as the harness.
pub struct Fixture {
    harness: Harness<'static, App>,
    helper: Arc<Helper>,
    _runtime: Runtime,
}

impl Fixture {
    pub const fn harness(&mut self) -> &mut Harness<'static, App> {
        &mut self.harness
    }
}

impl LockSource for Helper {
    fn locks(&self) -> Result<BTreeMap<UserName, bool>, LockProblem> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(BTreeMap::from([
            (name("ada"), true),
            (name("grace"), false),
            (name("nico"), false),
        ]))
    }
}

fn name(text: &str) -> UserName {
    UserName::new(text).unwrap()
}

fn user(text: &str, uid: u32, gecos: &str) -> User {
    User {
        name: name(text),
        uid,
        gid: uid,
        gecos: gecos.to_owned(),
        home: format!("/home/{text}"),
        shell: "/usr/bin/fish".to_owned(),
    }
}

fn group(text: &str, gid: u32, members: &[&str]) -> Group {
    Group {
        name: GroupName::new(text).unwrap(),
        gid,
        members: members.iter().map(|member| name(member)).collect(),
    }
}

/// `nico` logged in and admin, `ada` and `grace` as people, `http` as a
/// service.
fn system() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(Accounts {
        users: vec![
            user("ada", 1001, "Ada Lovelace"),
            user("grace", 1002, "Grace Hopper"),
            user("http", 33, ""),
            user("nico", 1000, "Nico"),
        ],
        groups: vec![
            group("ada", 1001, &[]),
            group("docker", 970, &["nico"]),
            group("grace", 1002, &[]),
            group("http", 33, &[]),
            group("nico", 1000, &[]),
            group("video", 1500, &["ada", "grace"]),
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

pub fn users() -> Fixture {
    let mut fixture = restored(ChangeSet::default());
    let harness = fixture.harness();
    harness
        .get_by_role_and_label(Role::Tab, "Users and Groups")
        .click();
    step_until(harness, "the lock flags", |harness| {
        harness.query_by_label("Locked").is_some()
    });
    settle(harness);
    fixture
}

/// The app on its start page after the first read, with `changes` as the
/// queue an earlier run left.
fn restored(changes: ChangeSet) -> Fixture {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();
    let helper = Arc::new(Helper::default());
    let locks = Arc::<Helper>::clone(&helper);
    let probed = system();
    let mut harness = window(HEIGHT).build_eframe(move |cc| {
        let probe = move || (probed.clone(), Vec::new());
        App::new(
            cc,
            Setup {
                queue: Queue::restored_in_memory(changes),
                snapshot: Some(snapshot::spawn(
                    &handle,
                    toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
                    probe,
                )),
                users: Some(UsersSetup {
                    runtime: handle.clone(),
                    locks,
                }),
                ..Setup::default()
            },
        )
    });

    first_read(&mut harness);

    Fixture {
        harness,
        helper,
        _runtime: runtime,
    }
}

/// The window asks the root helper program for the flags a restored lock
/// needs, also while the Users page is closed.
#[test]
fn a_restored_lock_reads_the_flags_before_apply() {
    let mut changes = ChangeSet::default();
    changes.want::<AccountLocks>(name("grace"), Toggle::On, &Snapshot::default());
    let mut fixture = restored(changes);
    let harness = fixture.harness();

    step_until(harness, "Apply after the flags", |harness| {
        !harness.get_by_label("Apply").accesskit_node().is_disabled()
    });

    assert_eq!(fixture.helper.0.load(Ordering::SeqCst), 1);
}

#[test]
fn the_helper_is_asked_once_and_a_locked_account_offers_its_unlock() {
    let mut fixture = users();
    let harness = fixture.harness();

    harness.get_by_label("More actions for ada").click();
    harness.run();

    assert!(harness.query_by_label("Queue unlock").is_some());
    assert_eq!(fixture.helper.0.load(Ordering::SeqCst), 1);
}

#[test]
fn a_queued_lock_waits_in_the_queue_and_shows_on_the_row() {
    let mut fixture = users();
    let harness = fixture.harness();

    harness.get_by_label("More actions for grace").click();
    harness.run();
    harness.get_by_label("Queue lock").click();
    harness.run();

    assert!(harness.query_by_label("1 change queued").is_some());
    assert!(harness.query_by_label("change queued").is_some());
}

#[test]
fn a_new_group_shows_as_new_before_it_is_applied() {
    let mut fixture = users();
    let harness = fixture.harness();

    harness
        .get_by_role_and_label(Role::RadioButton, "Groups")
        .click();
    harness.run();
    harness.get_by_label("Add group…").click();
    harness.run();
    type_into(harness, "Group name", "games");
    harness.run();
    harness.get_by_label("Queue group").click();
    harness.run();

    assert!(harness.query_by_label("New").is_some());
    assert!(harness.query_by_label("Don't add games").is_some());
}
