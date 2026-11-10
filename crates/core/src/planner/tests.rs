use std::collections::BTreeSet;

use toolbox_plan::{PlannedAction, RootAction};

use super::plan;
use crate::domain::{Domain, Stage, Staged, Summary, Toggle};
use crate::domains::{
    EnabledUnits, EnabledUserUnits, Groups, InstalledPackages, Packages, SystemUnitMasks,
    SystemUnits, UserUnits,
};
use crate::test_support::{membership, nina_in, package, sided, unit};
use crate::{ChangeSet, Snapshot, SummaryCode};

/// A domain defined outside the core, as a later feature would: it plugs
/// into the change set, the views and the planner without core changes.
struct Keyring;

impl Domain for Keyring {
    type Item = String;
    type State = ();
    type Wish = String;

    const ID: &'static str = "keyring";
    const STAGE: Stage = Stage::new(Stage::REPOSITORIES.rank() + 10);

    fn is_satisfied(_item: &String, _wish: &String, (): &()) -> bool {
        false
    }

    fn plan(_changes: &[(&String, &String)]) -> Vec<Staged> {
        vec![Staged::at(
            Self::STAGE,
            PlannedAction::Root(RootAction::RefreshKeyring),
        )]
    }

    named_key!();

    fn summary(_item: &String, wish: &String) -> Summary {
        Summary::of(SummaryCode::RefreshKeyring, wish)
    }
}

/// Two domains at one stage: the id decides, whatever the queue order was.
struct Mirrors;

impl Domain for Mirrors {
    type Item = String;
    type State = ();
    type Wish = String;

    const ID: &'static str = "a-mirrors";
    const STAGE: Stage = Keyring::STAGE;

    fn is_satisfied(_item: &String, _wish: &String, (): &()) -> bool {
        false
    }

    fn plan(_changes: &[(&String, &String)]) -> Vec<Staged> {
        vec![
            Staged::at(
                Self::STAGE,
                PlannedAction::Root(RootAction::RemoveDatabaseLock),
            ),
            Staged::at(
                Self::STAGE,
                PlannedAction::Root(RootAction::CleanPackageCache),
            ),
        ]
    }

    named_key!();

    fn summary(_item: &String, wish: &String) -> Summary {
        Summary::of(SummaryCode::RefreshKeyring, wish)
    }
}

#[test]
fn nothing_queued_plans_nothing() {
    assert!(plan(&ChangeSet::default()).is_empty());
}

#[test]
fn packages_are_batched_into_one_removal_and_one_install() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::from([package("vlc")])));
    let mut changes = ChangeSet::default();
    changes.want::<Packages>(package("obs-studio"), Toggle::On, &snapshot);
    changes.want::<Packages>(package("mpv"), Toggle::On, &snapshot);
    changes.want::<Packages>(package("vlc"), Toggle::Off, &snapshot);

    assert_eq!(
        sided(&changes),
        [
            "root: pacman -Rns --noconfirm -- vlc",
            "root: pacman -S --needed --noconfirm -- mpv obs-studio",
        ]
    );
}

#[test]
fn units_start_after_their_packages_and_groups() {
    let snapshot = Snapshot::default();
    let mut changes = ChangeSet::default();
    changes.want::<UserUnits>(unit("psd.service"), Toggle::On, &snapshot);
    changes.want::<SystemUnits>(unit("docker.socket"), Toggle::On, &snapshot);
    changes.want::<Groups>(membership("nico", "docker"), Toggle::On, &snapshot);
    changes.want::<Packages>(package("docker"), Toggle::On, &snapshot);

    assert_eq!(
        sided(&changes),
        [
            "root: pacman -S --needed --noconfirm -- docker",
            "root: gpasswd -a nico docker",
            "root: systemctl enable --now -- docker.socket",
            "user: systemctl --user enable --now -- psd.service",
        ]
    );
}

#[test]
fn disabling_and_leaving_are_planned_too() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(EnabledUnits(BTreeSet::from([unit("cups.socket")])));
    snapshot.insert(nina_in(&["lp"]));
    let mut changes = ChangeSet::default();
    changes.want::<SystemUnits>(unit("cups.socket"), Toggle::Off, &snapshot);
    changes.want::<Groups>(membership("nico", "lp"), Toggle::Off, &snapshot);

    assert_eq!(
        sided(&changes),
        [
            "root: systemctl disable --now -- cups.socket",
            "root: gpasswd -d nico lp",
        ]
    );
}

#[test]
fn a_new_domain_plugs_in_at_its_stage() {
    let snapshot = Snapshot::default();
    let mut changes = ChangeSet::default();
    changes.want::<Packages>(package("mpv"), Toggle::On, &snapshot);
    changes.want::<Keyring>("archlinux".to_owned(), "refresh".to_owned(), &snapshot);

    assert_eq!(
        sided(&changes),
        [
            "root: pacman-key --refresh-keys",
            "root: pacman -S --needed --noconfirm -- mpv"
        ]
    );
    assert_eq!(
        changes.views().first().map(|view| view.summary.clone()),
        Some(Summary::of(SummaryCode::RefreshKeyring, "refresh"))
    );
}

#[test]
fn a_unit_is_disabled_before_its_package_is_removed() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::from([package("docker")])));
    snapshot.insert(EnabledUnits(BTreeSet::from([unit("docker.socket")])));
    let mut changes = ChangeSet::default();
    changes.want::<Packages>(package("docker"), Toggle::Off, &snapshot);
    changes.want::<SystemUnits>(unit("docker.socket"), Toggle::Off, &snapshot);

    assert_eq!(
        sided(&changes),
        [
            "root: systemctl disable --now -- docker.socket",
            "root: pacman -Rns --noconfirm -- docker",
        ]
    );
}

#[test]
fn a_user_unit_is_disabled_before_its_package_is_removed() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::from([package("psd")])));
    snapshot.insert(EnabledUserUnits(BTreeSet::from([unit("psd.service")])));
    let mut changes = ChangeSet::default();
    changes.want::<Packages>(package("psd"), Toggle::Off, &snapshot);
    changes.want::<UserUnits>(unit("psd.service"), Toggle::Off, &snapshot);

    assert_eq!(
        sided(&changes),
        [
            "user: systemctl --user disable --now -- psd.service",
            "root: pacman -Rns --noconfirm -- psd",
        ]
    );
}

#[test]
fn one_domain_tears_down_early_and_switches_on_at_its_stage() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(EnabledUnits(BTreeSet::from([unit("cups.socket")])));
    let mut changes = ChangeSet::default();
    changes.want::<SystemUnits>(unit("cups.socket"), Toggle::Off, &snapshot);
    changes.want::<SystemUnits>(unit("docker.socket"), Toggle::On, &snapshot);
    changes.want::<Packages>(package("docker"), Toggle::On, &snapshot);

    assert_eq!(
        sided(&changes),
        [
            "root: systemctl disable --now -- cups.socket",
            "root: pacman -S --needed --noconfirm -- docker",
            "root: systemctl enable --now -- docker.socket",
        ]
    );
}

#[test]
fn a_unit_is_masked_after_it_is_disabled_and_unmasked_before_it_is_enabled() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(EnabledUnits(BTreeSet::from([unit("cups.socket")])));
    let mut changes = ChangeSet::default();
    changes.want::<SystemUnitMasks>(unit("cups.socket"), Toggle::On, &snapshot);
    changes.want::<SystemUnits>(unit("cups.socket"), Toggle::Off, &snapshot);
    changes.want::<SystemUnitMasks>(unit("sshd.service"), Toggle::Off, &snapshot);
    changes.want::<SystemUnits>(unit("sshd.service"), Toggle::On, &snapshot);

    assert_eq!(
        sided(&changes),
        [
            "root: systemctl disable --now -- cups.socket",
            "root: systemctl mask -- cups.socket",
            "root: systemctl unmask -- sshd.service",
            "root: systemctl enable --now -- sshd.service",
        ]
    );
}

#[test]
fn inside_one_stage_the_domain_id_decides_then_the_order_the_domain_gave() {
    let empty = Snapshot::default();
    let mut keyring_first = ChangeSet::default();
    keyring_first.want::<Keyring>("archlinux".to_owned(), "refresh".to_owned(), &empty);
    keyring_first.want::<Mirrors>("all".to_owned(), "rank".to_owned(), &empty);
    let mut mirrors_first = ChangeSet::default();
    mirrors_first.want::<Mirrors>("all".to_owned(), "rank".to_owned(), &empty);
    mirrors_first.want::<Keyring>("archlinux".to_owned(), "refresh".to_owned(), &empty);

    let expected = [
        "root: rm /var/lib/pacman/db.lck  # only while pacman is not running",
        "root: paccache -ruk0",
        "root: pacman-key --refresh-keys",
    ];

    assert_eq!(sided(&keyring_first), expected);
    assert_eq!(sided(&mirrors_first), expected);
}
