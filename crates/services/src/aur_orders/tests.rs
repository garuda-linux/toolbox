mod conflicts;

use std::collections::BTreeSet;

use toolbox_core::domains::{AurPackages, AurWish, BuildOrder, InstalledPackages};
use toolbox_core::features::packages::aur::{AurNames, toggle};
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_exec::RouteProblem;
use toolbox_plan::PackageName;
use toolbox_state::aur::order::parse_order;
use toolbox_state::aur::{AurRoute, RouteKind};
use toolbox_state::packages::PackageGraph;
use toolbox_state::testing::aur::order::{
    AUR_SYNC_VOTE_STDOUT, FIREFOX_STDOUT, MISSING_STDOUT, OCTOPI_GIT_STDOUT,
};
use toolbox_system::testing::aur_orders::FakeAurOrders;

use super::{NoOrder, Orders, fill, needing, read_orders, route_text_id};
use crate::queue::Queue;

fn package(name: &str) -> PackageName {
    PackageName::new(name).unwrap()
}

fn octopi_git() -> FakeAurOrders {
    FakeAurOrders::new(vec![("octopi-git", Ok(OCTOPI_GIT_STDOUT))])
}

/// The outcome of the only package asked.
fn only(orders: Orders) -> Result<BuildOrder, NoOrder> {
    let (_, outcome) = orders.into_iter().next().unwrap();
    outcome
}

#[test]
fn one_package_gets_its_order_its_run_and_the_installed_names_of_it() {
    let installed = BTreeSet::from([package("qt6-base"), package("not-in-the-order")]);
    let source = octopi_git();

    let built = only(
        read_orders(
            &[package("octopi-git")],
            &source,
            &installed,
            Ok(()),
            0x1a2b,
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(built.order, parse_order(OCTOPI_GIT_STDOUT).unwrap());
    assert_eq!(built.run.as_str(), "1a2b");
    assert_eq!(built.installed, BTreeSet::from([package("qt6-base")]));
}

#[test]
fn two_packages_of_one_call_share_one_run() {
    let source = FakeAurOrders::new(vec![
        ("octopi-git", Ok(OCTOPI_GIT_STDOUT)),
        ("aur-sync-vote", Ok(AUR_SYNC_VOTE_STDOUT)),
    ]);

    let orders = read_orders(
        &[package("octopi-git"), package("aur-sync-vote")],
        &source,
        &BTreeSet::new(),
        Ok(()),
        7,
    )
    .unwrap();

    let runs: Vec<String> = orders
        .iter()
        .map(|(_, outcome)| outcome.as_ref().unwrap().run.as_str().to_owned())
        .collect();

    assert_eq!(runs, ["7", "7"]);
    assert_eq!(source.asked(), 2);
}

#[test]
fn a_missing_line_gives_its_names_and_no_order() {
    let source = FakeAurOrders::new(vec![("no-such-package-toolbox", Ok(MISSING_STDOUT))]);

    let outcome = only(
        read_orders(
            &[package("no-such-package-toolbox")],
            &source,
            &BTreeSet::new(),
            Ok(()),
            1,
        )
        .unwrap(),
    );

    assert!(
        matches!(&outcome, Err(NoOrder::Missing(names)) if names.len() == 1),
        "{outcome:?}"
    );
}

#[test]
fn a_refused_route_gives_its_reason_to_every_package_and_asks_paru_for_none() {
    let source = FakeAurOrders::new(vec![("octopi-git", Ok(OCTOPI_GIT_STDOUT))]);

    let orders = read_orders(
        &[package("octopi-git"), package("aur-sync-vote")],
        &source,
        &BTreeSet::new(),
        Err(RouteProblem::NoAccount(1000)),
        1,
    )
    .unwrap();

    assert_eq!(orders.len(), 2);
    assert!(orders.iter().all(|(_, outcome)| matches!(
        outcome,
        Err(NoOrder::Route(problem)) if matches!(**problem, RouteProblem::NoAccount(1000))
    )));
    assert_eq!(source.asked(), 0);
}

#[test]
fn a_failing_paru_gives_its_reason() {
    let source = FakeAurOrders::new(vec![("octopi-git", Err("paru is not installed"))]);

    let outcome = only(
        read_orders(
            &[package("octopi-git")],
            &source,
            &BTreeSet::new(),
            Ok(()),
            1,
        )
        .unwrap(),
    );

    assert!(matches!(outcome, Err(NoOrder::Unread(_))), "{outcome:?}");
}

#[test]
fn an_order_that_does_not_name_the_package_as_a_build_gives_no_order() {
    let source = FakeAurOrders::new(vec![("firefox", Ok(FIREFOX_STDOUT))]);

    let outcome =
        only(read_orders(&[package("firefox")], &source, &BTreeSet::new(), Ok(()), 1).unwrap());

    assert!(matches!(outcome, Err(NoOrder::NotNamed)), "{outcome:?}");
}

#[test]
fn the_minted_run_is_valid_for_the_extreme_counters() {
    for counter in [0, u64::MAX] {
        let source = octopi_git();

        let built = only(
            read_orders(
                &[package("octopi-git")],
                &source,
                &BTreeSet::new(),
                Ok(()),
                counter,
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(built.run.as_str(), format!("{counter:x}"));
    }
}

/// A queue of `names` as the split path queues them, the route accepted
/// and the AUR's copy read.
fn queued(names: &[&str]) -> Queue {
    let installed = BTreeSet::from([package("paru")]);

    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(installed));
    snapshot.insert(PackageGraph::default());
    snapshot.insert(AurRoute::Accepted);
    snapshot.insert(AurNames::default());

    let mut changes = ChangeSet::default();
    for name in names {
        toggle(&package(name), Toggle::On, &snapshot, &mut changes).unwrap();
    }

    Queue::in_memory(changes)
}

fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::from([package("paru")])));
    snapshot.insert(PackageGraph::default());
    snapshot.insert(AurRoute::Accepted);
    snapshot.insert(AurNames::default());
    snapshot
}

fn wish(queue: &Queue, name: &str) -> Option<AurWish> {
    queue.changes().wish::<AurPackages>(&package(name)).cloned()
}

#[test]
fn a_fill_gives_the_octopi_build_its_order_and_the_queue_is_ready() {
    let mut queue = queued(&["octopi-git"]);
    let source = octopi_git();
    let orders = read_orders(
        &[package("octopi-git")],
        &source,
        &BTreeSet::new(),
        Ok(()),
        7,
    )
    .unwrap();

    let missing = fill(&mut queue, &snapshot(), orders);

    assert!(missing.is_empty());
    assert!(matches!(
        wish(&queue, "octopi-git"),
        Some(AurWish::Built(_))
    ));
    assert_eq!(queue.ready(), Ok(()));
}

#[test]
fn a_missing_answer_keeps_its_build_waiting_and_gives_the_reason() {
    let mut queue = queued(&["no-such-package-toolbox"]);
    let source = FakeAurOrders::new(vec![("no-such-package-toolbox", Ok(MISSING_STDOUT))]);
    let orders = read_orders(
        &[package("no-such-package-toolbox")],
        &source,
        &BTreeSet::new(),
        Ok(()),
        7,
    )
    .unwrap();

    let missing = fill(&mut queue, &snapshot(), orders);

    assert!(matches!(
        missing.as_slice(),
        [(name, NoOrder::Missing(_))] if *name == package("no-such-package-toolbox")
    ));
    assert_eq!(
        wish(&queue, "no-such-package-toolbox"),
        Some(AurWish::Waiting)
    );
    assert_eq!(
        queue.ready(),
        Err(toolbox_core::features::Refusal::OrderWaits {
            names: vec![package("no-such-package-toolbox")]
        })
    );
}

#[test]
fn a_second_fill_replaces_the_order_and_the_run_of_every_built_wish() {
    let mut queue = queued(&["octopi-git", "aur-sync-vote"]);
    let snapshot = snapshot();
    let source = FakeAurOrders::new(vec![
        ("octopi-git", Ok(OCTOPI_GIT_STDOUT)),
        ("aur-sync-vote", Ok(AUR_SYNC_VOTE_STDOUT)),
    ]);

    let names = [package("octopi-git"), package("aur-sync-vote")];

    let first = read_orders(&names, &source, &BTreeSet::new(), Ok(()), 1).unwrap();

    fill(&mut queue, &snapshot, first);
    let second = read_orders(&names, &source, &BTreeSet::new(), Ok(()), 2).unwrap();
    fill(&mut queue, &snapshot, second);

    let runs: Vec<String> = names
        .iter()
        .filter_map(|name| match wish(&queue, name.as_str()) {
            Some(AurWish::Built(built)) => Some(built.run.as_str().to_owned()),
            _ => None,
        })
        .collect();

    assert_eq!(runs, ["2", "2"]);
}

#[test]
fn the_queue_is_held_while_one_build_has_no_order() {
    let mut queue = queued(&["octopi-git", "aur-sync-vote"]);

    let snapshot = snapshot();
    let source = FakeAurOrders::new(vec![("octopi-git", Ok(OCTOPI_GIT_STDOUT))]);
    let names = [package("octopi-git"), package("aur-sync-vote")];

    let orders = read_orders(&names, &source, &BTreeSet::new(), Ok(()), 1).unwrap();
    let missing = fill(&mut queue, &snapshot, orders);

    assert!(matches!(
        missing.as_slice(),
        [(name, NoOrder::Unread(_))] if *name == package("aur-sync-vote")
    ));
    assert!(queue.ready().is_err());
    assert_eq!(
        needing(queue.changes()),
        [package("aur-sync-vote"), package("octopi-git")]
    );
}

#[test]
fn an_order_alone_changes_nothing_on_disk() {
    let mut queue = queued(&["octopi-git"]);
    let revision = queue.revision();
    let orders = read_orders(
        &[package("octopi-git")],
        &octopi_git(),
        &BTreeSet::new(),
        Ok(()),
        7,
    )
    .unwrap();

    fill(&mut queue, &snapshot(), orders);

    assert_eq!(queue.revision(), revision);
}
#[test]
fn a_fill_never_touches_a_paru_build() {
    let mut snapshot = snapshot();
    snapshot.insert(AurRoute::Refused {
        kind: RouteKind::Linked,
        detail: String::new(),
    });
    let mut changes = ChangeSet::default();
    toggle(&package("octopi-git"), Toggle::On, &snapshot, &mut changes).unwrap();
    let mut queue = Queue::in_memory(changes);
    let orders = read_orders(
        &[package("octopi-git")],
        &octopi_git(),
        &BTreeSet::new(),
        Ok(()),
        7,
    )
    .unwrap();

    fill(&mut queue, &snapshot, orders);

    assert_eq!(wish(&queue, "octopi-git"), Some(AurWish::Paru));
}

#[test]
fn every_reason_and_route_text_is_in_both_languages() {
    use fluent_templates::LanguageIdentifier;
    use strum::IntoEnumIterator as _;
    use toolbox_state::aur::RouteKind;

    use crate::i18n::{FALLBACK, text_with_in};

    let german: LanguageIdentifier = "de-DE".parse().unwrap();
    let reasons = [
        NoOrder::Unread(toolbox_system::ProbeError::Unavailable),
        NoOrder::Route(std::sync::Arc::new(RouteProblem::NoAccount(1000))),
        NoOrder::Missing(Vec::new()),
        NoOrder::NotNamed,
    ];

    let mut ids: Vec<&str> = reasons.iter().map(NoOrder::text_id).collect();
    ids.extend(RouteKind::iter().map(route_text_id));

    for language in [FALLBACK, german] {
        for id in &ids {
            let text = text_with_in(&language, id, &[("name", "yay-bin".into())]);
            assert_ne!(text, *id, "{id} has no text in {language}");
        }

        let old_path = text_with_in(
            &language,
            "aur-old-path",
            &[("reason", "a folder is missing".into())],
        );

        assert_ne!(old_path, "aur-old-path", "no old path text in {language}");
    }
}
