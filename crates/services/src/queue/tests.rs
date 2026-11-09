use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use tempfile::TempDir;
use toolbox_core::domains::{DkmsBuilds, InstalledPackages, Packages};
use toolbox_core::{ChangeSet, Domain as _, Queued, Snapshot, Toggle};
use toolbox_plan::{DkmsBuild, DkmsModuleName, DkmsVersion, KernelRelease, PackageName};

use super::{NotKept, Queue, read};
use crate::testing::PATIENCE;
use crate::waking::Wake;

fn package(name: &str) -> PackageName {
    name.parse().unwrap()
}

fn two_packages() -> ChangeSet {
    let mut changes = ChangeSet::default();
    let snapshot = Snapshot::default();
    changes.want::<Packages>(package("firefox"), Toggle::On, &snapshot);
    changes.want::<Packages>(package("vlc"), Toggle::On, &snapshot);
    changes
}

/// Waits for the writer task, which runs on its own schedule.
fn wait_for(path: &Path, done: impl Fn(&Path) -> bool) {
    let until = Instant::now() + PATIENCE;
    while !done(path) {
        assert!(Instant::now() < until, "the queue file did not settle");
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn a_missing_file_is_an_empty_queue() {
    let folder = TempDir::new().unwrap();

    assert!(read(&folder.path().join("changes.json"), &Snapshot::default()).is_empty());
}

#[test]
fn an_unreadable_file_is_an_empty_queue() {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("changes.json");
    fs::write(&path, "not json").unwrap();

    assert!(read(&path, &Snapshot::default()).is_empty());
}

#[test]
fn views_list_every_queued_change() {
    let queue = Queue::in_memory(two_packages());

    let keys: Vec<String> = queue.views().into_iter().map(|view| view.key).collect();

    assert_eq!(keys, ["firefox", "vlc"]);
}

#[test]
fn removing_one_change_keeps_the_others() {
    let mut queue = Queue::in_memory(two_packages());

    queue.remove("packages", "firefox");

    assert_eq!(queue.changes().len(), 1);
}

#[test]
fn clearing_empties_the_queue() {
    let mut queue = Queue::in_memory(two_packages());

    queue.clear();

    assert!(queue.changes().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn every_change_is_written_and_read_back() {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("changes.json");
    fs::write(&path, "").unwrap();

    let mut queue = Queue::with_file(
        two_packages(),
        path.clone(),
        &tokio::runtime::Handle::current(),
        Wake::none(),
    );

    queue.remove("packages", "vlc");

    let written = path.clone();
    tokio::task::spawn_blocking(move || {
        wait_for(&written, |file| read(file, &Snapshot::default()).len() == 1);
    })
    .await
    .unwrap();

    let restored = read(&path, &Snapshot::default());
    assert_eq!(restored.views()[0].key, "firefox");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_empty_queue_removes_its_file() {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("changes.json");
    fs::write(&path, "").unwrap();

    let mut queue = Queue::with_file(
        two_packages(),
        path.clone(),
        &tokio::runtime::Handle::current(),
        Wake::none(),
    );

    queue.clear();

    tokio::task::spawn_blocking(move || wait_for(&path, |file| !file.exists()))
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn a_queue_that_cannot_be_written_tells_the_window() {
    let folder = TempDir::new().unwrap();
    let blocker = folder.path().join("not-a-folder");
    fs::write(&blocker, "").unwrap();
    let woken = Arc::new(AtomicBool::new(false));
    let told = Arc::clone(&woken);

    let mut queue = Queue::with_file(
        ChangeSet::default(),
        blocker.join("changes.json"),
        &tokio::runtime::Handle::current(),
        Wake::new(move || told.store(true, Ordering::SeqCst)),
    );

    assert!(queue.not_kept().is_none());

    queue.edit(&Snapshot::default(), |changes, snapshot| {
        changes.want::<Packages>(package("firefox"), Toggle::On, snapshot);
    });

    let until = Instant::now() + PATIENCE;
    let failure: NotKept = loop {
        if let Some(failure) = queue.not_kept() {
            break failure;
        }

        assert!(Instant::now() < until, "no failed write was told");
        tokio::time::sleep(Duration::from_millis(20)).await;
    };

    assert!(woken.load(Ordering::SeqCst));
    assert!(failure.to_string().contains("not-a-folder"), "{failure}");
    assert!(queue.not_kept().is_none(), "a failure is told once");
}

#[test]
fn the_plan_holds_one_step_per_queued_package() {
    let queue = Queue::in_memory(two_packages());

    assert_eq!(queue.plan().steps().len(), 1);
    assert_eq!(Queue::in_memory(ChangeSet::default()).plan().steps(), []);
}

#[test]
fn one_offs_are_dropped_and_desired_state_stays() {
    let mut changes = two_packages();
    let build = DkmsBuild {
        module: DkmsModuleName::new("zfs").unwrap(),
        version: DkmsVersion::new("2.3.1").unwrap(),
        kernel: KernelRelease::new("6.14.0-arch1-1").unwrap(),
    };

    changes.want::<DkmsBuilds>(build, Queued, &Snapshot::default());
    let mut queue = Queue::in_memory(changes);

    queue.sent(&queue.plan());
    queue.plan_started();
    queue.plan_finished(|_| true, Some(&|_| true));

    let domains: Vec<&str> = queue.views().into_iter().map(|view| view.domain).collect();
    assert_eq!(domains, [Packages::ID, Packages::ID]);
}

/// A ping or a log read from a page ends too, and leaves the queued work.
#[test]
fn the_end_of_another_plan_keeps_the_one_offs() {
    let mut changes = ChangeSet::default();
    let build = DkmsBuild {
        module: DkmsModuleName::new("zfs").unwrap(),
        version: DkmsVersion::new("2.3.1").unwrap(),
        kernel: KernelRelease::new("6.14.0-arch1-1").unwrap(),
    };

    changes.want::<DkmsBuilds>(build, Queued, &Snapshot::default());
    let mut queue = Queue::in_memory(changes);

    queue.plan_started();
    queue.plan_finished(|_| true, Some(&|_| true));

    assert_eq!(queue.views().len(), 1);
}

fn installed(names: &[&str]) -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(
        names.iter().map(|name| package(name)).collect(),
    ));
    snapshot
}

#[test]
fn a_snapshot_drops_what_the_system_already_fulfils() {
    let mut changes = ChangeSet::default();
    let unknown = Snapshot::default();
    changes.want::<Packages>(package("firefox"), Toggle::On, &unknown);
    changes.want::<Packages>(package("steam"), Toggle::Off, &unknown);
    changes.want::<Packages>(package("vlc"), Toggle::On, &unknown);
    let mut queue = Queue::in_memory(changes);

    queue.meet(&installed(&["firefox"]));

    let keys: Vec<String> = queue.views().into_iter().map(|view| view.key).collect();
    assert_eq!(keys, vec!["vlc".to_owned()]);
}

#[test]
fn a_snapshot_without_the_part_keeps_the_queue() {
    let mut queue = Queue::in_memory(two_packages());

    queue.meet(&Snapshot::default());

    assert_eq!(queue.changes().len(), 2);
}

#[test]
fn an_edit_sees_the_snapshot_and_changes_the_queue() {
    let mut queue = Queue::in_memory(ChangeSet::default());
    let snapshot = installed(&["firefox"]);

    let queued = queue.edit(&snapshot, |changes, snapshot| {
        changes.want::<Packages>(package("firefox"), Toggle::Off, snapshot);
        changes.want::<Packages>(package("firefox-esr"), Toggle::On, snapshot);
        changes.len()
    });

    assert_eq!(queued, 2);
    assert_eq!(queue.changes().len(), 2);
}

#[test]
fn the_revision_moves_with_each_change_and_only_then() {
    let mut queue = Queue::in_memory(two_packages());
    let start = queue.revision();

    queue.remove("packages", "nothing-queued");
    assert_eq!(queue.revision(), start);

    queue.remove("packages", "firefox");
    let removed = queue.revision();
    assert_ne!(removed, start);

    queue.edit(&Snapshot::default(), |changes, snapshot| {
        changes.want::<Packages>(package("gimp"), Toggle::On, snapshot);
    });
    assert_ne!(queue.revision(), removed);
}

#[test]
fn an_edit_from_a_command_joins_the_queue_against_the_snapshot() {
    let mut queue = Queue::in_memory(two_packages());
    let mut wanted = two_packages();
    wanted.remove::<Packages>(&package("vlc"));
    wanted.want::<Packages>(package("gimp"), Toggle::On, &Snapshot::default());
    wanted.want::<Packages>(package("steam"), Toggle::On, &Snapshot::default());
    let edit = super::edit::QueueEdit::between(&two_packages(), &wanted);
    let start = queue.revision();

    queue.take(
        &edit,
        &toolbox_core::domains::all().unwrap(),
        &installed(&["steam"]),
    );

    let keys: Vec<String> = queue.views().into_iter().map(|view| view.key).collect();
    assert_eq!(keys, ["firefox", "gimp"]);
    assert_ne!(queue.revision(), start);
}

#[test]
fn a_queue_written_by_a_command_reads_back() {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("changes.json");

    super::store(&path, &two_packages().to_saved()).unwrap();
    assert_eq!(read(&path, &Snapshot::default()).len(), 2);

    super::store(&path, &ChangeSet::default().to_saved()).unwrap();
    assert!(!path.exists());
}

/// The install failed, yet another program put firefox in place: the
/// plan did not finish, so the change waits for the next apply.
#[test]
fn a_change_whose_plan_failed_stays_when_the_system_shows_it() {
    let mut queue = Queue::in_memory(two_packages());

    queue.sent(&queue.plan());
    queue.plan_started();
    queue.plan_finished(|_| false, Some(&|_| false));
    queue.meet(&installed(&["firefox", "vlc"]));

    let failed: Vec<bool> = queue
        .views()
        .iter()
        .map(|change| change.outcome.is_failed())
        .collect();

    assert_eq!(failed, [true, true]);
}

/// A dry run skips every step, so it tells no change that it failed.
#[test]
fn a_dry_run_marks_no_change() {
    let mut queue = Queue::in_memory(two_packages());

    queue.sent(&queue.plan());
    queue.plan_started();
    queue.plan_finished(|_| false, None);
    queue.meet(&installed(&["firefox", "vlc"]));

    assert_eq!(queue.views(), []);
}
