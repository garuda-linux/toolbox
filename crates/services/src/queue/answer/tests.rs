use toolbox_core::domains::{AurPackages, InstalledPackages, Packages};
use toolbox_core::features::packages::aur::AurNames;
use toolbox_core::{ChangeSet, Queued, SavedChanges, Snapshot, Toggle};
use toolbox_instance::bus::queue::{Question, Refusal};

use super::super::Queue;
use super::super::edit::QueueEdit;

fn firefox() -> ChangeSet {
    let mut changes = ChangeSet::default();
    changes.want::<Packages>("firefox".parse().unwrap(), Toggle::On, &Snapshot::default());
    changes
}

#[test]
fn the_window_tells_its_queue_in_the_saved_form() {
    let mut queue = Queue::in_memory(firefox());

    let text = queue
        .answer(Question::Changes, false, &Snapshot::default())
        .unwrap();

    let saved: SavedChanges = serde_json::from_str(&text).unwrap();
    assert_eq!(saved, firefox().to_saved());
}

#[test]
fn an_edit_from_a_command_is_queued() {
    let mut queue = Queue::in_memory(ChangeSet::default());
    let edit = QueueEdit::between(&ChangeSet::default(), &firefox());

    queue
        .answer(Question::Edit(edit.to_text()), false, &Snapshot::default())
        .unwrap();

    assert_eq!(queue.changes().len(), 1);
}

#[test]
fn no_edit_while_the_window_applies_its_queue() {
    let mut queue = Queue::in_memory(ChangeSet::default());
    let edit = QueueEdit::between(&ChangeSet::default(), &firefox());

    let answer = queue.answer(Question::Edit(edit.to_text()), true, &Snapshot::default());

    assert_eq!(answer, Err(Refusal::Busy));
    assert!(queue.changes().is_empty());
}

#[test]
fn text_that_is_no_edit_leaves_the_queue_alone() {
    let mut queue = Queue::in_memory(firefox());

    let answer = queue.answer(
        Question::Edit(r#"{"remove":"everything"}"#.to_owned()),
        false,
        &Snapshot::default(),
    );

    assert_eq!(answer, Err(Refusal::Invalid));
    assert_eq!(queue.changes().len(), 1);
}

/// An AUR build over the bus waits for the AUR's copy, then goes when
/// the copy lacks its name, as the page would never have queued it.
#[test]
fn an_aur_build_the_copy_lacks_goes_once_the_copy_is_known() {
    let mut queue = Queue::in_memory(ChangeSet::default());
    let mut wanted = ChangeSet::default();
    for name in ["yay-bin", "not-in-aur"] {
        wanted.want::<AurPackages>(name.parse().unwrap(), Queued, &Snapshot::default());
    }

    let mut system = Snapshot::default();
    system.insert(InstalledPackages::default());
    let edit = QueueEdit::between(&ChangeSet::default(), &wanted);
    queue
        .answer(Question::Edit(edit.to_text()), false, &system)
        .unwrap();
    assert_eq!(queue.changes().len(), 2, "the builds wait for the copy");

    system.insert(AurNames(
        std::iter::once("yay-bin".parse().unwrap()).collect(),
    ));
    queue.meet(&system);

    assert_eq!(
        queue.changes().queued::<AurPackages>(),
        [("yay-bin".parse().unwrap(), Queued)]
    );
}
