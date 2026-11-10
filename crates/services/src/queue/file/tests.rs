use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use toolbox_core::domains::Packages;
use toolbox_core::testing::Queue as _;
use toolbox_core::{ChangeSet, RestoreProblem, SAVED_VERSION, Snapshot, Toggle};
use toolbox_plan::PackageName;

use super::{Restored, keep_aside, read};
use crate::queue::Queue;
use crate::testing::until;
use crate::waking::Wake;

fn firefox() -> ChangeSet {
    let mut changes = ChangeSet::default();
    let package = PackageName::new("firefox").unwrap();
    changes.want::<Packages>(package, Toggle::On, &Snapshot::default());
    changes
}

fn backup(path: &Path) -> PathBuf {
    path.with_file_name("changes.json.bak")
}

/// A shape change fails the typed parse, the version still names why.
#[test]
fn a_file_of_another_version_is_dropped_for_its_version() {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("changes.json");
    let text = format!("{{\"version\":{},\"queue\":[]}}", SAVED_VERSION + 1);
    fs::write(&path, text).unwrap();

    let (changes, problems) = read(&path, &Snapshot::default());

    assert!(changes.is_empty());
    assert_eq!(problems, [RestoreProblem::Version(SAVED_VERSION + 1)]);
}

#[test]
fn a_file_kept_aside_replaces_the_older_backup() {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("changes.json");
    fs::write(backup(&path), "older").unwrap();
    fs::write(&path, "newer").unwrap();

    keep_aside(&path).unwrap();

    assert!(!path.exists());
    assert_eq!(fs::read_to_string(backup(&path)).unwrap(), "newer");
}

#[test]
fn nothing_to_keep_aside_is_no_failure() {
    let folder = TempDir::new().unwrap();

    keep_aside(&folder.path().join("changes.json")).unwrap();
}

/// The first write after a lossy restore keeps what was dropped findable.
#[tokio::test(flavor = "multi_thread")]
async fn a_lossy_restore_keeps_the_old_file_before_the_first_write() {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("changes.json");
    fs::write(&path, "not json").unwrap();

    let mut queue = Queue::with_file(
        ChangeSet::default(),
        path.clone(),
        &tokio::runtime::Handle::current(),
        Wake::none(),
        Restored::Lossy,
    );

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "not json",
        "no write yet"
    );
    queue.edit(&Snapshot::default(), |changes, _| *changes = firefox());

    let written = path.clone();
    tokio::task::spawn_blocking(move || {
        until("the queue file did not settle", || {
            read(&written, &Snapshot::default()).0.len() == 1
        });
    })
    .await
    .unwrap();

    assert_eq!(fs::read_to_string(backup(&path)).unwrap(), "not json");
}

/// The window's close writes the queue itself, as the runtime may stop
/// before its task does.
#[test]
fn a_flush_writes_the_queue_at_once() {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("changes.json");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    // The runtime never runs, so its task writes nothing.
    let queue = Queue::with_file(
        firefox(),
        path.clone(),
        runtime.handle(),
        Wake::none(),
        Restored::Whole,
    );

    queue.flush().unwrap();

    assert_eq!(read(&path, &Snapshot::default()).0.len(), 1);
}

/// Started once more after a stop, then told: a writer that keeps
/// stopping would never keep the queue.
#[test]
fn a_writer_that_stops_twice_is_told_once() {
    let folder = TempDir::new().unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    let handle = runtime.handle().clone();
    let mut queue = Queue::with_file(
        firefox(),
        folder.path().join("changes.json"),
        &handle,
        Wake::none(),
        Restored::Whole,
    );

    // Its tasks go with the runtime, and a task started later is dropped.
    drop(runtime);

    assert!(queue.not_kept().is_none(), "started once more");
    let told = queue.not_kept().expect("the second stop is told");
    assert!(told.to_string().contains("stopped"), "{told}");
    assert!(queue.not_kept().is_none(), "told once");
}
