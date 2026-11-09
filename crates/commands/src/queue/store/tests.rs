use tempfile::TempDir;
use toolbox_core::domains::Packages;
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_services::queue::edit::QueueEdit;

use super::{File, Store as _};

fn firefox() -> ChangeSet {
    let mut changes = ChangeSet::default();
    changes.want::<Packages>("firefox".parse().unwrap(), Toggle::On, &Snapshot::default());
    changes
}

#[test]
fn without_a_file_the_queue_is_empty() {
    let folder = TempDir::new().unwrap();
    let file = File {
        path: folder.path().join("changes.json"),
    };

    assert_eq!(file.saved().unwrap().changes, []);
    assert!(!file.in_window());
}

#[test]
fn an_edit_lands_in_the_file_and_its_folder_is_made() {
    let folder = TempDir::new().unwrap();
    let file = File {
        path: folder.path().join("state").join("changes.json"),
    };

    file.edit(&QueueEdit::between(&ChangeSet::default(), &firefox()))
        .unwrap();

    assert_eq!(file.saved().unwrap(), firefox().to_saved());

    file.edit(&QueueEdit::between(&firefox(), &ChangeSet::default()))
        .unwrap();
    assert!(!file.path.exists());
}

/// Two commands at once each read, change and write the file. The lock
/// makes the second wait, so neither change is lost.
#[test]
fn an_edit_waits_for_the_lock_of_another_command() {
    use std::time::Duration;

    use rustix::fs::{FlockOperation, flock};

    let folder = TempDir::new().unwrap();
    let path = folder.path().join("changes.json");
    let held = std::fs::File::create(folder.path().join("changes.json.lock")).unwrap();

    flock(&held, FlockOperation::LockExclusive).unwrap();

    let waiting = std::thread::Builder::new()
        .name("second command".to_owned())
        .spawn({
            let path = path.clone();

            move || {
                File { path }
                    .edit(&QueueEdit::between(&ChangeSet::default(), &firefox()))
                    .unwrap();
            }
        })
        .unwrap();

    std::thread::sleep(Duration::from_millis(200));
    assert!(!path.exists(), "the edit did not wait for the lock");

    drop(held);
    waiting.join().unwrap();

    assert_eq!(File { path }.saved().unwrap(), firefox().to_saved());
}
