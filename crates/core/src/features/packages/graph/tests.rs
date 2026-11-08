use std::collections::BTreeSet;

use toolbox_plan::PackageName;
use toolbox_state::packages::{InstallReason, LocalNode};

use super::{dependency_name, orphans, takes_along};

fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

fn node(text: &str, reason: InstallReason, depends: &[&str], provides: &[&str]) -> LocalNode {
    LocalNode {
        name: name(text),
        depends: depends.iter().map(|&dep| dep.to_owned()).collect(),
        provides: provides.iter().map(|&dep| dep.to_owned()).collect(),
        reason,
    }
}

fn explicit(text: &str, depends: &[&str]) -> LocalNode {
    node(text, InstallReason::Explicit, depends, &[])
}

fn dependency(text: &str, depends: &[&str]) -> LocalNode {
    node(text, InstallReason::Dependency, depends, &[])
}

fn names(texts: &[&str]) -> Vec<PackageName> {
    texts.iter().map(|text| name(text)).collect()
}

#[test]
fn a_dependency_string_names_the_package_before_its_version() {
    assert_eq!(dependency_name("glibc>=2.40"), "glibc");
    assert_eq!(dependency_name("libfoo.so=1-64"), "libfoo.so");
    assert_eq!(dependency_name("python<3.14"), "python");
    assert_eq!(dependency_name("sh"), "sh");
}

#[test]
fn removal_takes_the_dependencies_nothing_else_needs() {
    let local = [
        explicit("gimp", &["babl>=0.1", "gegl"]),
        dependency("babl", &[]),
        dependency("gegl", &["babl"]),
        explicit("inkscape", &[]),
    ];

    assert_eq!(
        takes_along(&names(&["gimp"]), &local),
        names(&["babl", "gegl"])
    );
}

#[test]
fn a_dependency_another_package_still_needs_stays() {
    let local = [
        explicit("gimp", &["gegl", "glib2"]),
        dependency("gegl", &[]),
        dependency("glib2", &[]),
        explicit("gtk3", &["glib2"]),
    ];

    assert_eq!(takes_along(&names(&["gimp"]), &local), names(&["gegl"]));
}

#[test]
fn an_explicitly_installed_dependency_stays() {
    let local = [explicit("gimp", &["gegl"]), explicit("gegl", &[])];

    assert_eq!(takes_along(&names(&["gimp"]), &local).len(), 0);
}

#[test]
fn a_dependency_met_through_provides_goes_with_its_last_user() {
    let local = [
        explicit("ffmpeg", &["libva.so=2-64"]),
        node("libva", InstallReason::Dependency, &[], &["libva.so=2-64"]),
    ];

    assert_eq!(takes_along(&names(&["ffmpeg"]), &local), names(&["libva"]));
}

#[test]
fn packages_removed_together_do_not_keep_each_other() {
    let local = [
        explicit("a", &["shared"]),
        explicit("b", &["shared"]),
        dependency("shared", &[]),
    ];

    assert_eq!(takes_along(&names(&["a", "b"]), &local), names(&["shared"]));
    assert_eq!(takes_along(&names(&["a"]), &local).len(), 0);
}

#[test]
fn a_dependency_cycle_goes_as_a_whole() {
    let local = [
        explicit("app", &["left"]),
        dependency("left", &["right"]),
        dependency("right", &["left"]),
    ];

    assert_eq!(
        takes_along(&names(&["app"]), &local),
        names(&["left", "right"])
    );
}

#[test]
fn orphans_are_dependencies_nothing_requires() {
    let local = [
        explicit("gimp", &["gegl"]),
        dependency("gegl", &[]),
        dependency("leftover", &["libleft"]),
        dependency("libleft", &[]),
        explicit("unused", &[]),
    ];

    assert_eq!(
        orphans(&local),
        BTreeSet::from([name("leftover")]),
        "pacman -Qdt lists only the top of an unused chain"
    );
}

#[test]
fn a_package_required_through_what_it_provides_is_no_orphan() {
    let local = [
        explicit("bash-completion", &["sh"]),
        node("dash", InstallReason::Dependency, &[], &["sh"]),
    ];

    assert_eq!(orphans(&local).len(), 0);
}
