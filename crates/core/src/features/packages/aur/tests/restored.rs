//! AUR wishes that came from the saved file or the session bus, which no
//! click checked.

use std::collections::BTreeSet;

use toolbox_state::aur::AurRoute;

use super::super::{AurNames, recheck};
use super::{copy_with_conflicts, refused, with_route};
use crate::domains::{AurPackages, AurWish};
use crate::test_support::package;
use crate::{ChangeSet, Snapshot};

fn named(mut copy: AurNames, names: &[&str]) -> AurNames {
    copy.names = names
        .iter()
        .map(|name| package(name))
        .collect::<BTreeSet<_>>();
    copy
}

fn waiting(name: &str, snapshot: &Snapshot) -> ChangeSet {
    let mut changes = ChangeSet::default();
    changes.want::<AurPackages>(package(name), AurWish::Waiting, snapshot);
    changes
}

/// A click under a refused route queues paru's build. A wish that waits for
/// a build order there would hold Apply for good.
#[test]
fn a_waiting_wish_under_a_refused_route_becomes_a_paru_build() {
    let copy = named(AurNames::default(), &["octopi-git"]);
    let snapshot = with_route(&[], Some(refused()), copy);
    let mut changes = waiting("octopi-git", &snapshot);

    recheck(&mut changes, &snapshot);

    assert_eq!(
        changes.wish::<AurPackages>(&package("octopi-git")),
        Some(&AurWish::Paru)
    );
}

/// A click refuses a build that conflicts with an installed package, since
/// `pacman -U --noconfirm` declines the removal after the whole build.
#[test]
fn a_wish_that_conflicts_with_an_installed_package_is_dropped() {
    let copy = named(copy_with_conflicts("yay-bin", &["yay"]), &["yay-bin"]);
    let snapshot = with_route(&["yay"], Some(AurRoute::Accepted), copy);
    let mut changes = waiting("yay-bin", &snapshot);

    assert_eq!(recheck(&mut changes, &snapshot), Some(1));
    assert!(changes.is_empty());
}

#[test]
fn a_waiting_wish_the_route_accepts_stays() {
    let copy = named(AurNames::default(), &["octopi-git"]);
    let snapshot = with_route(&[], Some(AurRoute::Accepted), copy);
    let mut changes = waiting("octopi-git", &snapshot);

    assert_eq!(recheck(&mut changes, &snapshot), Some(0));
    assert_eq!(
        changes.wish::<AurPackages>(&package("octopi-git")),
        Some(&AurWish::Waiting)
    );
}
