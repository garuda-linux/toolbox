use std::collections::BTreeSet;

use toolbox_plan::StepId;

use crate::domains::{InstalledPackages, Packages};
use crate::test_support::{failed_at, outcomes, package, stopped_at};
use crate::{ChangeSet, Outcome, Snapshot, Toggle, planned};

fn installed(names: &[&str]) -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(
        names
            .iter()
            .map(|name| package(name))
            .collect::<BTreeSet<_>>(),
    ));
    snapshot
}

fn queued(names: &[&str]) -> ChangeSet {
    let mut changes = ChangeSet::default();
    for name in names {
        changes.want::<Packages>(package(name), Toggle::On, &Snapshot::default());
    }

    changes
}

#[test]
fn a_new_change_is_untried() {
    assert_eq!(
        outcomes(&queued(&["mpv"])),
        [("mpv".to_owned(), Outcome::Untried)]
    );
}

#[test]
fn every_step_ended_well_and_the_change_succeeded() {
    let mut changes = queued(&["mpv"]);
    let (_, owners) = planned(&changes);

    changes.finished(&owners, |_| true);

    assert_eq!(outcomes(&changes), [("mpv".to_owned(), Outcome::Succeeded)]);
}

#[test]
fn a_failed_change_stays_although_the_system_shows_its_wish() {
    let mut changes = queued(&["mpv"]);
    stopped_at(&mut changes, 1);

    changes.reconcile(&installed(&["mpv"]));

    assert_eq!(outcomes(&changes), [("mpv".to_owned(), failed_at(1))]);
}

#[test]
fn a_failed_change_leaves_after_an_apply_that_finished_its_plan() {
    let mut changes = queued(&["mpv"]);
    stopped_at(&mut changes, 1);
    let (_, owners) = planned(&changes);

    changes.finished(&owners, |_| true);
    changes.reconcile(&installed(&["mpv"]));

    assert!(changes.is_empty());
}

#[test]
fn a_new_wish_for_the_item_makes_it_untried() {
    let mut changes = queued(&["mpv"]);
    stopped_at(&mut changes, 1);

    changes.want::<Packages>(package("mpv"), Toggle::On, &Snapshot::default());

    assert_eq!(outcomes(&changes), [("mpv".to_owned(), Outcome::Untried)]);
}

#[test]
fn a_failed_change_taken_back_is_gone() {
    let mut changes = queued(&["mpv"]);
    stopped_at(&mut changes, 1);

    assert!(changes.remove_key("packages", "mpv"));
    assert!(changes.is_empty());
}

/// Queued while the plan ran, so the plan's end tells it nothing.
#[test]
fn a_change_the_plan_did_not_carry_out_keeps_its_outcome() {
    let mut changes = ChangeSet::default();
    changes.want::<crate::domains::Dns>(
        crate::Single,
        toolbox_plan::DnsProvider::Cloudflare,
        &Snapshot::default(),
    );
    let (_, owners) = planned(&changes);
    changes.want::<Packages>(package("mpv"), Toggle::On, &Snapshot::default());

    changes.finished(&owners, |_| false);

    assert_eq!(
        outcomes(&changes),
        [
            ("mpv".to_owned(), Outcome::Untried),
            ("dns".to_owned(), failed_at(1)),
        ]
    );
}

#[test]
fn the_first_step_that_did_not_end_well_is_named() {
    let mut changes = queued(&["mpv", "vlc"]);
    changes.want::<crate::domains::Dns>(
        crate::Single,
        toolbox_plan::DnsProvider::Cloudflare,
        &Snapshot::default(),
    );
    let (plan, owners) = planned(&changes);
    assert_eq!(plan.steps().len(), 2);

    changes.finished(&owners, |step| step == StepId::new(1));

    assert_eq!(
        outcomes(&changes),
        [
            ("mpv".to_owned(), Outcome::Succeeded),
            ("vlc".to_owned(), Outcome::Succeeded),
            ("dns".to_owned(), failed_at(2)),
        ]
    );
}
