use toolbox_plan::PackageName;
use toolbox_state::packages::Protection;

use super::{PROTECTED, ProtectedCatalog};
use crate::CatalogError;

fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap()
}

#[test]
fn each_package_keeps_its_reason() {
    let text = "[[package]]\nname = \"sudo\"\nwhy = \"root\"\n\n\
                [[package]]\nname = \"pacman\"\nwhy = \"package-manager\"\n";

    let catalog = ProtectedCatalog::parse(text, "protected.toml").unwrap();

    assert_eq!(
        catalog.packages().0.get(&name("sudo")),
        Some(&Protection::Root)
    );
    assert_eq!(
        catalog.packages().0.get(&name("pacman")),
        Some(&Protection::PackageManager)
    );
}

#[test]
fn a_package_listed_twice_is_refused() {
    let text = "[[package]]\nname = \"sudo\"\nwhy = \"root\"\n\n\
                [[package]]\nname = \"sudo\"\nwhy = \"login\"\n";

    assert!(matches!(
        ProtectedCatalog::parse(text, "protected.toml"),
        Err(CatalogError::Invalid {
            source_name: _,
            id: _,
            problem: _
        })
    ));
}

#[test]
fn an_unknown_reason_is_refused() {
    let text = "[[package]]\nname = \"sudo\"\nwhy = \"handy\"\n";

    assert!(matches!(
        ProtectedCatalog::parse(text, "protected.toml"),
        Err(CatalogError::Syntax {
            source_name: _,
            message: _
        })
    ));
}

/// The shipped file keeps what the system needs to boot, log in and get
/// root.
#[test]
fn the_shipped_list_keeps_the_core_of_the_system() {
    let catalog = ProtectedCatalog::load(&crate::source_dir()).unwrap();
    let kept = &catalog.packages().0;

    for package in [
        "sudo",
        "systemd",
        "polkit",
        "pacman",
        "linux-firmware",
        "util-linux",
    ] {
        assert!(kept.contains_key(&name(package)), "{package}");
    }

    assert_eq!(PROTECTED, "protected.toml");
}
