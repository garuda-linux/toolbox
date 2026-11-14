use toolbox_core::domains::{InstalledPackages, Packages};
use toolbox_core::testing::Queue as _;
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_plan::PackageName;

use super::super::Queue;
use super::on_copy;

fn package(name: &str) -> PackageName {
    name.parse().unwrap()
}

fn clicked(before: &ChangeSet, name: &str, wish: Toggle, snapshot: &Snapshot) -> ChangeSet {
    on_copy(before, |changes| {
        changes.want::<Packages>(package(name), wish, snapshot);
    })
    .0
}

fn keys(queue: &Queue) -> Vec<String> {
    queue.views().into_iter().map(|view| view.key).collect()
}

#[test]
fn a_click_queues_and_takes_back_as_it_did_on_the_copy() {
    let mut queue = Queue::in_memory(ChangeSet::default());
    let none = Snapshot::default();

    queue.take_click(clicked(queue.changes(), "firefox", Toggle::On, &none));
    assert_eq!(keys(&queue), ["firefox"]);

    let (back, taken) = on_copy(queue.changes(), |changes| {
        changes.remove_key("packages", "firefox")
    });

    assert!(taken);
    queue.take_click(back);
    assert_eq!(keys(&queue), Vec::<String>::new());
}

/// The bus path runs every guard again, which would hold Apply until
/// each part is read once more. A click met its page's rules already.
#[test]
fn a_click_keeps_the_met_guards() {
    let mut queue = Queue::in_memory(ChangeSet::default());

    queue.take_click(clicked(
        queue.changes(),
        "firefox",
        Toggle::On,
        &Snapshot::default(),
    ));

    assert_eq!(queue.ready(), Ok(()));
    assert_eq!(queue.holding().count(), 0);
}

#[test]
fn a_click_counts_as_a_new_queue_only_when_it_changed_it() {
    let mut queue = Queue::in_memory(ChangeSet::default());
    let before = queue.revision();

    queue.take_click(queue.changes().clone());
    assert_eq!(queue.revision(), before);

    queue.take_click(clicked(
        queue.changes(),
        "firefox",
        Toggle::On,
        &Snapshot::default(),
    ));
    assert_ne!(queue.revision(), before);
}

/// The click saw the system meet the wish, so nothing was queued.
#[test]
fn a_click_the_system_meets_queues_nothing() {
    let mut queue = Queue::in_memory(ChangeSet::default());
    let mut installed = Snapshot::default();
    installed.insert(InstalledPackages([package("firefox")].into()));

    queue.take_click(clicked(queue.changes(), "firefox", Toggle::On, &installed));

    assert_eq!(keys(&queue), Vec::<String>::new());
}
