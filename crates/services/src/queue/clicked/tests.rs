use toolbox_core::domains::{InstalledPackages, Packages};
use toolbox_core::testing::Queue as _;
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_plan::PackageName;

use super::super::Queue;
use super::super::edit::QueueEdit;

fn package(name: &str) -> PackageName {
    name.parse().unwrap()
}

fn clicked(before: &ChangeSet, name: &str, wish: Toggle) -> QueueEdit {
    QueueEdit::of_click(before, |changes| {
        changes.want::<Packages>(package(name), wish, &Snapshot::default());
    })
    .0
}

fn keys(queue: &Queue) -> Vec<String> {
    queue.views().into_iter().map(|view| view.key).collect()
}

#[test]
fn a_page_edit_queues_and_takes_back_as_its_click_did() {
    let mut queue = Queue::in_memory(ChangeSet::default());

    let add = clicked(queue.changes(), "firefox", Toggle::On);
    queue.apply_edit(&add, &Snapshot::default());
    assert_eq!(keys(&queue), ["firefox"]);

    let (back, taken) = QueueEdit::of_click(queue.changes(), |changes| {
        changes.remove_key("packages", "firefox")
    });

    assert!(taken);
    queue.apply_edit(&back, &Snapshot::default());
    assert_eq!(keys(&queue), Vec::<String>::new());
}

/// The bus path runs every guard again, which would hold Apply until
/// each part is read once more. A click met its page's rules already.
#[test]
fn a_page_edit_keeps_the_met_guards() {
    let mut queue = Queue::in_memory(ChangeSet::default());

    queue.apply_edit(
        &clicked(queue.changes(), "firefox", Toggle::On),
        &Snapshot::default(),
    );

    assert_eq!(queue.ready(), Ok(()));
    assert_eq!(queue.holding().count(), 0);
}

#[test]
fn a_page_edit_counts_as_a_new_queue_only_when_it_changed_it() {
    let mut queue = Queue::in_memory(ChangeSet::default());
    let before = queue.revision();

    queue.apply_edit(&QueueEdit::removing(Vec::new()), &Snapshot::default());
    assert_eq!(queue.revision(), before);

    queue.apply_edit(
        &clicked(queue.changes(), "firefox", Toggle::On),
        &Snapshot::default(),
    );
    assert_ne!(queue.revision(), before);
}

/// The wish meets the system by the time App takes it, so nothing stays.
#[test]
fn a_page_edit_the_system_meets_queues_nothing() {
    let mut queue = Queue::in_memory(ChangeSet::default());
    let mut installed = Snapshot::default();
    installed.insert(InstalledPackages([package("firefox")].into()));

    queue.apply_edit(&clicked(queue.changes(), "firefox", Toggle::On), &installed);

    assert_eq!(keys(&queue), Vec::<String>::new());
}

/// A failed change keeps its outcome in the copy, so an edit of another
/// change leaves it alone.
#[test]
fn a_click_edit_names_only_what_the_click_changed() {
    let mut queue = Queue::in_memory(ChangeSet::default());
    queue.apply_edit(
        &clicked(queue.changes(), "firefox", Toggle::On),
        &Snapshot::default(),
    );

    let edit = clicked(queue.changes(), "vlc", Toggle::On);

    assert_eq!(edit.remove, []);
    assert_eq!(edit.add.len(), 1);
}
