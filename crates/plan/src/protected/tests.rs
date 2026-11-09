use std::collections::BTreeSet;

use super::{PROTECTED, Protection, protection};
use crate::PackageName;

#[test]
fn every_entry_is_a_package_name() {
    for (name, _) in PROTECTED {
        assert!(PackageName::new(*name).is_ok(), "{name}");
    }
}

#[test]
fn no_entry_appears_twice() {
    let names: BTreeSet<&str> = PROTECTED.iter().map(|(name, _)| *name).collect();

    assert_eq!(names.len(), PROTECTED.len());
}

#[test]
fn a_listed_package_is_kept_for_its_reason() {
    let sudo = PackageName::new("sudo").unwrap();

    assert_eq!(protection(&sudo), Some(Protection::Root));
}

#[test]
fn the_boot_manager_and_systemd_libraries_are_kept() {
    for name in ["efibootmgr", "systemd-libs"] {
        let package = PackageName::new(name).unwrap();

        assert_eq!(protection(&package), Some(Protection::Boot), "{name}");
    }
}

#[test]
fn an_unlisted_package_is_not_kept() {
    let gimp = PackageName::new("gimp").unwrap();

    assert_eq!(protection(&gimp), None);
}

#[test]
fn each_reason_names_its_text() {
    assert_eq!(Protection::PackageManager.as_str(), "package-manager");
    assert_eq!(Protection::Boot.as_str(), "boot");
}
