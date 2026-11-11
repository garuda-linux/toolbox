//! A saved file of every domain, as the app wrote it. A change to a
//! domain's serde shape fails here, instead of dropping the users' queued
//! changes with a log line.

use crate::{ChangeSet, SavedChanges, Snapshot};

/// Each written by `queue add` of the command in
/// `crates/commands/src/parity.rs`, against its fake system.
const SAVED: &[(&str, &str)] = &[
    (
        "account-locks",
        include_str!("../fixtures/account-locks.json"),
    ),
    (
        "aur-packages",
        include_str!("../fixtures/aur-packages.json"),
    ),
    (
        "dkms-modules",
        include_str!("../fixtures/dkms-modules.json"),
    ),
    ("dns", include_str!("../fixtures/dns.json")),
    ("full-names", include_str!("../fixtures/full-names.json")),
    (
        "garuda-update-flags",
        include_str!("../fixtures/garuda-update-flags.json"),
    ),
    ("groups", include_str!("../fixtures/groups.json")),
    (
        "grub-settings",
        include_str!("../fixtures/grub-settings.json"),
    ),
    ("hblock", include_str!("../fixtures/hblock.json")),
    (
        "home-folders",
        include_str!("../fixtures/home-folders.json"),
    ),
    (
        "loader-settings",
        include_str!("../fixtures/loader-settings.json"),
    ),
    ("locales", include_str!("../fixtures/locales.json")),
    (
        "login-shells",
        include_str!("../fixtures/login-shells.json"),
    ),
    (
        "module-blacklist",
        include_str!("../fixtures/module-blacklist.json"),
    ),
    ("new-groups", include_str!("../fixtures/new-groups.json")),
    ("operations", include_str!("../fixtures/operations.json")),
    ("packages", include_str!("../fixtures/packages.json")),
    (
        "plymouth-theme",
        include_str!("../fixtures/plymouth-theme.json"),
    ),
    (
        "removed-groups",
        include_str!("../fixtures/removed-groups.json"),
    ),
    (
        "removed-users",
        include_str!("../fixtures/removed-users.json"),
    ),
    (
        "sdboot-kernel",
        include_str!("../fixtures/sdboot-kernel.json"),
    ),
    (
        "system-locale",
        include_str!("../fixtures/system-locale.json"),
    ),
    ("unit-masks", include_str!("../fixtures/unit-masks.json")),
    ("units", include_str!("../fixtures/units.json")),
    (
        "user-configs",
        include_str!("../fixtures/user-configs.json"),
    ),
    (
        "user-unit-masks",
        include_str!("../fixtures/user-unit-masks.json"),
    ),
    ("user-units", include_str!("../fixtures/user-units.json")),
    (
        "wifi-backend",
        include_str!("../fixtures/wifi-backend.json"),
    ),
];

#[test]
fn every_domain_has_a_saved_fixture() {
    let ids: Vec<&str> = SAVED.iter().map(|(id, _text)| *id).collect();

    assert_eq!(ids, crate::domains::all().unwrap().ids());
}

/// Nothing the empty snapshot fulfils, so every change comes back and is
/// saved as the same bytes.
#[test]
fn every_saved_fixture_restores_and_saves_the_same() {
    let domains = crate::domains::all().unwrap();

    for (id, text) in SAVED {
        let saved: SavedChanges = serde_json::from_str(text).unwrap();
        let mut changes = ChangeSet::default();

        let problems = domains.restore(&mut changes, &saved, &Snapshot::default());

        assert_eq!(problems, [], "{id}");
        assert!(changes.holds(id), "{id}");
        assert_eq!(
            serde_json::to_string_pretty(&changes.to_saved()).unwrap() + "\n",
            *text,
            "{id}"
        );
    }
}
