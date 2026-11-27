use super::*;

fn waiting_build(item: &str) -> ChangeSet {
    let mut changes = ChangeSet::default();
    changes.want::<AurPackages>(package(item), AurWish::Waiting, &Snapshot::default());
    changes
}

#[test]
fn a_waiting_build_order_refuses_the_plan_with_its_package() {
    let queue = checked(waiting_build("yay-bin"));

    assert!(matches!(
        queue.plan(),
        Err(Refusal::OrderWaits { names }) if names == [package("yay-bin")]
    ));
}

/// The restored queue is held by its `waiting` wish, and a second wish of
/// the same queue, which is a Paru install, does not unlock it.
#[test]
fn a_queue_restored_with_a_waiting_build_is_held_until_the_build_is_taken_out() {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("changes.json");
    fs::write(
        &path,
        r#"{"version":6,"changes":[{"domain":"aur-packages","item":"yay-bin","wish":"waiting"},{"domain":"aur-packages","item":"paru-bin","wish":"paru"}]}"#,
    )
    .unwrap();
    let mut parts = package_parts();
    parts.insert(AurNames {
        names: BTreeSet::from([package("yay-bin"), package("paru-bin")]),
        ..AurNames::default()
    });

    let (changes, problems) = read(&path, &parts);
    assert!(problems.is_empty(), "{problems:?}");
    let mut queue = Queue::restored_in_memory(changes);
    queue.meet(&parts);

    assert!(matches!(
        queue.plan(),
        Err(Refusal::OrderWaits { names }) if names == [package("yay-bin")]
    ));

    queue.remove(AurPackages::ID, &AurPackages::key(&package("yay-bin")));
    queue.plan().unwrap();
}
