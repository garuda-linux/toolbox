use std::collections::BTreeSet;

use toolbox_core::domains::BuildOrder;
use toolbox_exec::RouteProblem;
use toolbox_plan::PackageName;
use toolbox_state::aur::order::parse_order;
use toolbox_state::testing::aur::order::{
    AUR_SYNC_VOTE_STDOUT, FIREFOX_STDOUT, MISSING_STDOUT, OCTOPI_GIT_STDOUT,
};
use toolbox_system::testing::aur_orders::FakeAurOrders;

use super::{NoOrder, Orders, read_orders};

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
