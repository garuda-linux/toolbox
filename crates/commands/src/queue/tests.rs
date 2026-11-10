use std::cell::RefCell;

use toolbox_cli::Exit;
use toolbox_core::domains::{InstalledPackages, Packages, all};
use toolbox_core::testing::Queue as _;
use toolbox_core::{ChangeSet, SavedChanges, Snapshot, Summary, Toggle, planned};
use toolbox_instance::bus::queue::Refusal;
use toolbox_services::queue::edit::QueueEdit;

use super::store::{Problem, Store};
use super::{Failed, add_change, clear_all, remove_change, restored, settle};
use crate::change::Refused;

/// A queue kept in memory, in the window or not, that may refuse edits.
struct Kept {
    saved: RefCell<SavedChanges>,
    window: bool,
    refusal: Option<Refusal>,
    edits: RefCell<Vec<QueueEdit>>,
}

impl Kept {
    fn new(names: &[&str]) -> Self {
        Self {
            saved: RefCell::new(packages(names).to_saved()),
            window: false,
            refusal: None,
            edits: RefCell::new(Vec::new()),
        }
    }

    fn keys(&self) -> Vec<String> {
        restored(&self.saved.borrow(), &Snapshot::default())
            .views()
            .into_iter()
            .map(|view| view.key)
            .collect()
    }
}

impl Store for Kept {
    fn in_window(&self) -> bool {
        self.window
    }

    fn saved(&self) -> Result<SavedChanges, Problem> {
        Ok(self.saved.borrow().clone())
    }

    fn edit(&self, edit: &QueueEdit) -> Result<(), Problem> {
        if let Some(refusal) = self.refusal {
            return Err(Problem::Window(refusal));
        }

        let mut changes = restored(&self.saved.borrow(), &Snapshot::default());
        edit.apply(&mut changes, &all().unwrap(), &Snapshot::default());
        *self.saved.borrow_mut() = changes.to_saved();
        self.edits.borrow_mut().push(edit.clone());
        Ok(())
    }
}

fn packages(names: &[&str]) -> ChangeSet {
    let mut changes = ChangeSet::default();
    for name in names {
        changes.want::<Packages>(name.parse().unwrap(), Toggle::On, &Snapshot::default());
    }

    changes
}

/// A click on a package against `snapshot`, as a change command makes it.
fn want(
    name: &'static str,
    wish: Toggle,
    snapshot: &Snapshot,
) -> impl FnOnce(&mut ChangeSet) -> Result<(), Refused> {
    move |changes| {
        changes.want::<Packages>(name.parse().unwrap(), wish, snapshot);
        Ok(())
    }
}

fn words(summary: &Summary) -> String {
    summary.code.as_str().to_owned()
}

#[test]
fn an_added_change_joins_the_queue_beside_the_others() {
    let kept = Kept::new(&["firefox"]);

    let waiting = add_change(
        &kept,
        &Snapshot::default(),
        want("vlc", Toggle::On, &Snapshot::default()),
    )
    .unwrap();

    assert_eq!(waiting, 2);
    assert_eq!(kept.keys(), ["firefox", "vlc"]);
    assert_eq!(kept.edits.borrow()[0].remove, []);
}

#[test]
fn switching_a_queued_change_back_takes_it_out_as_a_click_would() {
    // Not installed, so taking the install back leaves nothing to do.
    let mut nothing = Snapshot::default();
    nothing.insert(InstalledPackages(std::collections::BTreeSet::new()));
    let kept = Kept::new(&["firefox"]);
    add_change(&kept, &nothing, want("firefox", Toggle::Off, &nothing)).unwrap();

    assert_eq!(kept.keys(), Vec::<String>::new());
}

#[test]
fn a_change_the_queue_has_already_is_nothing_to_do() {
    let kept = Kept::new(&["firefox"]);

    let failed = add_change(
        &kept,
        &Snapshot::default(),
        want("firefox", Toggle::On, &Snapshot::default()),
    )
    .unwrap_err();

    assert_eq!(failed, Failed::Nothing);
    assert_eq!(failed.exit(), Exit::Success);
    assert!(kept.edits.borrow().is_empty());
}

#[test]
fn a_busy_window_keeps_its_queue_and_says_so() {
    let mut kept = Kept::new(&["firefox"]);
    kept.window = true;
    kept.refusal = Some(Refusal::Busy);

    let failed = add_change(
        &kept,
        &Snapshot::default(),
        want("vlc", Toggle::On, &Snapshot::default()),
    )
    .unwrap_err();

    assert_eq!(failed, Failed::Store(Problem::Window(Refusal::Busy)));
    assert_eq!(failed.exit(), Exit::Failure);
    assert_eq!(kept.keys(), ["firefox"]);
}

#[test]
fn a_change_goes_by_its_number_or_its_name() {
    let kept = Kept::new(&["firefox", "gimp", "vlc"]);

    assert_eq!(
        remove_change(&kept, &Snapshot::default(), "2", words).unwrap(),
        2
    );
    assert_eq!(kept.keys(), ["firefox", "vlc"]);

    remove_change(&kept, &Snapshot::default(), "vlc", words).unwrap();
    assert_eq!(kept.keys(), ["firefox"]);

    let unknown = remove_change(&kept, &Snapshot::default(), "7", words).unwrap_err();
    assert_eq!(unknown.exit(), Exit::Unavailable);
}

#[test]
fn clear_takes_every_change_back() {
    let kept = Kept::new(&["firefox", "vlc"]);

    assert_eq!(clear_all(&kept).unwrap(), 2);
    assert_eq!(kept.keys(), Vec::<String>::new());
    assert_eq!(clear_all(&kept).unwrap_err(), Failed::Nothing);
}

#[test]
fn after_an_apply_the_queue_keeps_only_what_the_system_lacks() {
    let kept = Kept::new(&["firefox", "vlc"]);
    let mut after = Snapshot::default();
    after.insert(InstalledPackages(["firefox".parse().unwrap()].into()));

    let (_, owners) = planned(&restored(&kept.saved().unwrap(), &Snapshot::default()));
    settle(&kept, &after, &owners, |_| true, &[]).unwrap();

    assert_eq!(kept.keys(), ["vlc"]);
}

/// The install stopped, yet firefox is there: the change stays, marked,
/// until an apply finishes its plan.
#[test]
fn after_a_failed_apply_the_queue_keeps_what_the_system_shows() {
    let kept = Kept::new(&["firefox", "vlc"]);
    let mut after = Snapshot::default();
    after.insert(InstalledPackages(["firefox".parse().unwrap()].into()));
    let (_, owners) = planned(&restored(&kept.saved().unwrap(), &Snapshot::default()));

    settle(&kept, &after, &owners, |_| false, &[]).unwrap();
    let failed: Vec<bool> = restored(&kept.saved().unwrap(), &after)
        .views()
        .iter()
        .map(|change| change.outcome.is_failed())
        .collect();

    assert_eq!(failed, [true, true]);

    settle(&kept, &after, &owners, |_| true, &[]).unwrap();
    assert_eq!(kept.keys(), ["vlc"]);
}
