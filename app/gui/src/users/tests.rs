use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::runtime::Runtime;
use toolbox_core::features::users::run::{PageRun, RunKind};
use toolbox_plan::UserName;
use toolbox_services::locks::Locks;
use toolbox_system::pkexec::Refusal;
use toolbox_system::{LockProblem, LockSource};

use super::{Users, UsersSetup};
use crate::testing::until;

struct Helper {
    refuse: bool,
    reads: AtomicUsize,
}

impl LockSource for Helper {
    fn locks(&self) -> Result<BTreeMap<UserName, bool>, LockProblem> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        if self.refuse {
            return Err(LockProblem::Refused(Refusal::Dismissed));
        }

        Ok(BTreeMap::from([(name("ada"), true)]))
    }
}

fn name(text: &str) -> UserName {
    UserName::new(text).unwrap()
}

fn users(refuse: bool) -> (Runtime, Arc<Helper>, Users) {
    let runtime = Runtime::new().unwrap();
    let helper = Arc::new(Helper {
        refuse,
        reads: AtomicUsize::new(0),
    });

    let setup = UsersSetup {
        runtime: runtime.handle().clone(),
        locks: Arc::<Helper>::clone(&helper),
    };

    let users = Users::new(Some(setup), egui::Context::default());
    (runtime, helper, users)
}

fn settle(users: &mut Users) {
    until("the flags never changed", || users.take());
}

#[test]
fn nothing_is_asked_before_the_page_opens() {
    let (_runtime, helper, mut users) = users(false);

    users.plan_finished(true);
    std::thread::sleep(Duration::from_millis(50));

    assert!(!users.take());
    assert_eq!(*users.locks(), Locks::NotAsked);
    assert_eq!(helper.reads.load(Ordering::SeqCst), 0);
}

#[test]
fn opening_asks_once_and_the_flags_reach_the_snapshot() {
    let (_runtime, helper, mut users) = users(false);

    users.open();
    assert_eq!(*users.locks(), Locks::Loading);
    settle(&mut users);
    users.open();

    assert_eq!(users.locks().of(&name("ada")), Some(true));
    assert!(users.flags().is_some());
    assert_eq!(helper.reads.load(Ordering::SeqCst), 1);
}

#[test]
fn a_refused_read_leaves_the_flags_unknown() {
    let (_runtime, _helper, mut users) = users(true);

    users.open();
    until("the locks were never read", || {
        users.take();
        *users.locks() != Locks::Loading
    });

    assert_eq!(*users.locks(), Locks::Refused(Refusal::Dismissed));
    assert_eq!(users.flags(), None);
}

#[test]
fn a_run_takes_the_outcome_of_the_next_plan() {
    let mut users = Users::default();

    users.started(RunKind::Password, name("ada"));
    users.plan_started();
    users.plan_finished(false);

    assert_eq!(
        *users.run(),
        PageRun::Failed(RunKind::Password, name("ada"))
    );

    users.dismiss_run();

    assert_eq!(*users.run(), PageRun::None);
}

#[test]
fn a_refused_run_is_not_reported_as_done() {
    let mut users = Users::default();

    users.started(RunKind::Password, name("ada"));
    users.plan_refused();
    users.plan_finished(true);

    assert_eq!(*users.run(), PageRun::None);
}
