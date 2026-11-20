use toolbox_plan::PackageName;
use toolbox_text::OutsideText;

use super::{AurBuild, AurOrder, InvalidOrder, Role, parse_order};

/// `paru -Po firefox` (paru 2.2.1) on the Garuda Lima VM: repo packages only.
const FIREFOX: &str = include_str!("fixtures/lima/paru-po-firefox.txt");
/// `paru -Po --aur aur-sync-vote`: build-only repo packages, then the build.
const AUR_SYNC_VOTE: &str = include_str!("fixtures/lima/paru-po-aur-sync-vote.txt");
/// `paru -Po --aur octopi-git`: an AUR dependency built before the target.
const OCTOPI_GIT: &str = include_str!("fixtures/lima/paru-po-octopi-git.txt");
/// `paru -Po --aur nvidia-470xx-utils opencl-nvidia-470xx`: two packages of
/// one pkgbase, each on its own line.
const NVIDIA_SPLIT: &str = include_str!("fixtures/lima/paru-po-nvidia-470xx-split.txt");

fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap()
}

fn build(role: Role, base: &str, package: &str) -> AurBuild {
    AurBuild {
        role,
        base: name(base),
        name: name(package),
    }
}

fn names(order: &AurOrder, role: Role) -> Vec<&str> {
    order
        .repo
        .iter()
        .filter(|step| step.role == role)
        .map(|step| step.name.as_str())
        .collect()
}

#[test]
fn a_repo_target_comes_after_its_dependencies() {
    let order = parse_order(FIREFOX).unwrap();

    assert_eq!(names(&order, Role::Dependency).len(), 98);
    assert_eq!(names(&order, Role::Target), ["firefox"]);
    assert_eq!(
        order.repo.last().map(|step| step.name.as_str()),
        Some("firefox")
    );
    assert!(order.builds.is_empty() && order.missing.is_empty());
}

#[test]
fn build_only_packages_are_told_apart() {
    let order = parse_order(AUR_SYNC_VOTE).unwrap();

    assert_eq!(names(&order, Role::Dependency).len(), 12);
    assert_eq!(names(&order, Role::Make).first(), Some(&"python-packaging"));
    assert_eq!(names(&order, Role::Make).len(), 10);
    assert_eq!(
        order.builds,
        [build(Role::Target, "aur-sync-vote", "aur-sync-vote")]
    );
}

#[test]
fn an_aur_dependency_builds_first() {
    let order = parse_order(OCTOPI_GIT).unwrap();

    assert_eq!(
        order.builds,
        [
            build(
                Role::Dependency,
                "alpm_octopi_utils-git",
                "alpm_octopi_utils-git"
            ),
            build(Role::Target, "octopi-git", "octopi-git"),
        ]
    );
}

#[test]
fn packages_of_one_pkgbase_keep_their_base() {
    let order = parse_order(NVIDIA_SPLIT).unwrap();

    assert_eq!(
        order.builds,
        [
            build(Role::Target, "nvidia-470xx-utils", "nvidia-470xx-utils"),
            build(Role::Target, "nvidia-470xx-utils", "opencl-nvidia-470xx"),
        ]
    );
}

#[test]
fn a_missing_package_is_named() {
    let order = parse_order(crate::testing::aur::order::MISSING_STDOUT).unwrap();

    assert_eq!(
        order
            .missing
            .iter()
            .map(OutsideText::as_str)
            .collect::<Vec<_>>(),
        ["no-such-package-toolbox"]
    );
    assert!(order.repo.is_empty() && order.builds.is_empty());
}

/// The octopi-git capture with its build line cut short.
#[test]
fn a_cut_line_fails_the_read() {
    let text = OCTOPI_GIT.replace("AUR TARGET octopi-git octopi-git", "AUR TARGET octopi-git");

    assert!(matches!(parse_order(&text), Err(InvalidOrder::Line(_))));
}

/// An unknown role fails the read rather than dropping its line.
#[test]
fn an_unknown_role_fails_the_read() {
    let build = OCTOPI_GIT.replace(
        "AUR DEP alpm_octopi_utils-git",
        "AUR CHECK alpm_octopi_utils-git",
    );

    let repo = OCTOPI_GIT.replace("REPO MAKE extra vala", "REPO CHECK extra vala");

    assert!(matches!(parse_order(&build), Err(InvalidOrder::Line(_))));
    assert!(matches!(parse_order(&repo), Err(InvalidOrder::Line(_))));
}

/// The capture with a name pacman would refuse.
#[test]
fn an_invalid_name_fails_the_read() {
    let text = FIREFOX.replace("REPO TARGET extra firefox", "REPO TARGET extra fire;fox");

    assert!(matches!(parse_order(&text), Err(InvalidOrder::Name(_))));
}

/// A newer paru may print lines of another kind.
#[test]
fn an_unknown_line_is_left_out() {
    let text = format!("NOTE something new\n{FIREFOX}");

    assert_eq!(parse_order(&text), parse_order(FIREFOX));
}
