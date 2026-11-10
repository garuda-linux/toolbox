use std::collections::BTreeSet;

use super::ChangeSet;
use crate::Snapshot;
use crate::domain::{Domain, Stage, Staged, Summary, Toggle};
use crate::domains::{
    EnabledUnits, EnabledUserUnits, Groups, InstalledPackages, Packages, SystemUnits, UserUnits,
};
use crate::test_support::{membership, nina_in, package, unit, views};

/// Work that leaves no state behind, like the maintenance operations: it
/// needs no probe and is done once an apply succeeded.
struct Chores;

impl Domain for Chores {
    type Item = String;
    type State = ();
    type Wish = Toggle;

    const ID: &'static str = "chores";
    const ONE_OFF: bool = true;
    const STAGE: Stage = Stage::REPOSITORIES;

    fn state(_snapshot: &Snapshot) -> Option<&()> {
        Some(&())
    }

    fn is_satisfied(_item: &String, wish: &Toggle, (): &()) -> bool {
        !wish.is_on()
    }

    fn plan(_changes: &[(&String, &Toggle)]) -> Vec<Staged> {
        Vec::new()
    }

    named_key!();

    fn summary(item: &String, wish: &Toggle) -> Summary {
        Summary::toggle(*wish, "chore-on", "chore-off", item)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BadWish;

impl serde::Serialize for BadWish {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(serde::ser::Error::custom("never savable"))
    }
}

impl<'de> serde::Deserialize<'de> for BadWish {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        serde::de::IgnoredAny::deserialize(deserializer).map(|_| Self)
    }
}

struct BadSave;

impl Domain for BadSave {
    type Item = String;
    type State = ();
    type Wish = BadWish;

    const ID: &'static str = "bad-save";
    const STAGE: Stage = Stage::REPOSITORIES;

    fn state(_snapshot: &Snapshot) -> Option<&()> {
        Some(&())
    }

    fn is_satisfied(_item: &String, _wish: &BadWish, (): &()) -> bool {
        false
    }

    fn plan(_changes: &[(&String, &BadWish)]) -> Vec<Staged> {
        Vec::new()
    }

    named_key!();

    fn summary(item: &String, _wish: &BadWish) -> Summary {
        Summary::of("bad-save", item)
    }
}

fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::from([package("vlc")])));
    snapshot.insert(EnabledUnits(BTreeSet::from([unit("cups.socket")])));
    snapshot.insert(EnabledUserUnits::default());
    snapshot.insert(nina_in(&["wheel"]));
    snapshot
}

#[test]
fn a_wish_that_differs_from_the_system_is_kept() {
    let mut changes = ChangeSet::default();

    changes.want::<Packages>(package("mpv"), Toggle::On, &snapshot());

    assert_eq!(changes.len(), 1);
    assert_eq!(changes.wish::<Packages>(&package("mpv")), Some(&Toggle::On));
}

#[test]
fn a_wish_the_system_already_fulfils_is_dropped() {
    let mut changes = ChangeSet::default();

    changes.want::<Packages>(package("vlc"), Toggle::On, &snapshot());
    changes.want::<SystemUnits>(unit("cups.socket"), Toggle::On, &snapshot());

    assert!(changes.is_empty());
}

#[test]
fn toggling_back_cancels_the_change() {
    let mut changes = ChangeSet::default();

    changes.want::<Packages>(package("vlc"), Toggle::Off, &snapshot());
    changes.want::<Packages>(package("vlc"), Toggle::On, &snapshot());

    assert!(changes.is_empty());
}

#[test]
fn without_a_probe_result_every_wish_is_kept() {
    let mut changes = ChangeSet::default();

    changes.want::<Packages>(package("vlc"), Toggle::On, &Snapshot::default());

    assert_eq!(changes.len(), 1);
}

#[test]
fn reconciling_drops_what_an_apply_already_did() {
    let mut changes = ChangeSet::default();
    changes.want::<Packages>(package("mpv"), Toggle::On, &snapshot());
    changes.want::<Packages>(package("obs-studio"), Toggle::On, &snapshot());
    let mut after = snapshot();
    after.insert(InstalledPackages(BTreeSet::from([
        package("vlc"),
        package("mpv"),
    ])));

    changes.reconcile(&after);

    let keys: Vec<String> = changes.views().into_iter().map(|view| view.key).collect();
    assert_eq!(keys, ["obs-studio"]);
}

#[test]
fn the_review_page_can_remove_a_change_by_domain_and_key() {
    let mut changes = ChangeSet::default();
    changes.want::<Packages>(package("mpv"), Toggle::On, &snapshot());
    changes.want::<Groups>(membership("nico", "docker"), Toggle::On, &snapshot());

    assert!(changes.remove_key("packages", "mpv"));
    assert!(!changes.remove_key("packages", "mpv"));
    assert!(!changes.remove_key("units", "nico docker"));
    assert_eq!(changes.len(), 1);
}

#[test]
fn user_units_are_separate_from_system_units() {
    let mut changes = ChangeSet::default();

    changes.want::<UserUnits>(unit("cups.socket"), Toggle::On, &snapshot());

    assert_eq!(
        changes.len(),
        1,
        "cups.socket is enabled for the system, not the user"
    );
}

#[test]
fn views_come_in_plan_order_with_their_summary() {
    let mut changes = ChangeSet::default();
    changes.want::<SystemUnits>(unit("docker.socket"), Toggle::On, &snapshot());
    changes.want::<Packages>(package("docker"), Toggle::On, &snapshot());

    assert_eq!(
        views(&changes),
        [
            (
                "packages",
                "docker".to_owned(),
                Summary::of("package-install", "docker")
            ),
            (
                "units",
                "docker.socket".to_owned(),
                Summary::of("unit-enable", "docker.socket")
            ),
        ]
    );
}

#[test]
fn a_domain_without_a_probe_can_say_where_its_state_comes_from() {
    let mut changes = ChangeSet::default();

    changes.want::<Chores>("sweep".to_owned(), Toggle::On, &Snapshot::default());
    changes.want::<Chores>("dust".to_owned(), Toggle::Off, &Snapshot::default());
    assert_eq!(changes.len(), 1);

    changes.want::<Chores>("sweep".to_owned(), Toggle::Off, &Snapshot::default());
    assert!(changes.is_empty());
}

#[test]
fn one_off_changes_survive_a_reconcile_but_not_a_successful_apply() {
    let mut changes = ChangeSet::default();
    changes.want::<Chores>("sweep".to_owned(), Toggle::On, &snapshot());
    changes.want::<Packages>(package("mpv"), Toggle::On, &snapshot());

    changes.reconcile(&snapshot());
    assert_eq!(changes.len(), 2);

    crate::test_support::all_steps_succeeded(&mut changes);

    let keys: Vec<String> = changes.views().into_iter().map(|view| view.key).collect();
    assert_eq!(keys, ["mpv"]);
}

#[test]
fn an_unsavable_change_is_skipped_without_failing_the_save() {
    let mut changes = ChangeSet::default();
    changes.want::<BadSave>("sweep".to_owned(), BadWish, &Snapshot::default());
    changes.want::<Packages>(package("mpv"), Toggle::On, &snapshot());

    assert_eq!(changes.len(), 2);

    let saved = changes.to_saved();

    assert_eq!(saved.changes.len(), 1);
    assert_eq!(saved.changes[0].domain, "packages");
}
