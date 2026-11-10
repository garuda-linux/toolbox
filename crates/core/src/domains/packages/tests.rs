use super::{InstalledPackages, Packages};
use crate::SummaryCode;
use crate::domain::{Domain as _, Summary, Toggle};
use crate::test_support::{package, packages};

fn installed(names: &[&str]) -> InstalledPackages {
    InstalledPackages(packages(names))
}

#[test]
fn a_package_is_satisfied_when_installed_as_wished() {
    let system = installed(&["mpv"]);

    assert!(Packages::is_satisfied(
        &package("mpv"),
        &Toggle::On,
        &system
    ));
    assert!(Packages::is_satisfied(
        &package("vlc"),
        &Toggle::Off,
        &system
    ));
    assert!(!Packages::is_satisfied(
        &package("vlc"),
        &Toggle::On,
        &system
    ));
}

#[test]
fn package_changes_are_named_like_their_package() {
    assert_eq!(Packages::key(&package("mpv")), "mpv");
    assert_eq!(
        Packages::summary(&package("mpv"), &Toggle::Off),
        Summary::of(SummaryCode::PackageRemove, "mpv")
    );
}
