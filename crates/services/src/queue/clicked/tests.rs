use std::collections::BTreeSet;

use toolbox_core::domains::{AurPackages, AurWish, BuildOrder, InstalledPackages, Packages};
use toolbox_core::testing::Queue as _;
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_plan::{BuildRun, PackageName};
use toolbox_state::aur::order::parse_order;
use toolbox_state::testing::aur::order::OCTOPI_GIT_STDOUT;

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

#[test]
fn a_click_on_another_row_keeps_a_built_aur_wish_as_it_was() {
    let order = parse_order(OCTOPI_GIT_STDOUT).unwrap();
    let built = AurWish::Built(BuildOrder {
        order,
        run: BuildRun::new("1a2b").unwrap(),
        installed: BTreeSet::new(),
    });

    let none = Snapshot::default();
    let mut queue = Queue::in_memory(ChangeSet::default());
    let mut changes = ChangeSet::default();
    changes.want::<AurPackages>(package("octopi-git"), built.clone(), &none);
    queue.take_click(changes);

    queue.take_click(clicked(queue.changes(), "firefox", Toggle::On, &none));

    assert_eq!(keys(&queue), ["firefox", "octopi-git"]);
    assert_eq!(
        queue.changes().wish::<AurPackages>(&package("octopi-git")),
        Some(&built)
    );
}

#[test]
fn a_built_aur_wish_saves_as_waiting_so_a_click_on_its_copy_saves_nothing() {
    let order = parse_order(OCTOPI_GIT_STDOUT).unwrap();
    let built = AurWish::Built(BuildOrder {
        order,
        run: BuildRun::new("1a2b").unwrap(),
        installed: BTreeSet::new(),
    });

    let none = Snapshot::default();
    let mut changes = ChangeSet::default();
    changes.want::<AurPackages>(package("octopi-git"), built, &none);
    let mut queue = Queue::in_memory(ChangeSet::default());
    queue.take_click(changes);
    let revision = queue.revision();

    let (same, _answer) = on_copy(queue.changes(), |_changes| ());
    queue.take_click(same);

    assert_eq!(queue.revision(), revision);
}
