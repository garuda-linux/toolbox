mod guard;
mod recheck;
mod running;

use std::collections::{BTreeMap, BTreeSet};

use toolbox_plan::KernelRelease;

use super::{
    Kernel, KernelReleases, LinuxPackages, Listed, LocalPackage, Refusal, RepoPackage,
    install_missing_headers, kernels, lacks_headers, toggle,
};
use crate::domains::{InstalledPackages, Packages};
use crate::testing::package;
use crate::{ChangeSet, Snapshot, Toggle};

fn release(name: &str) -> KernelRelease {
    KernelRelease::new(name).unwrap()
}

fn repo(repo: &str, name: &str, version: &str) -> RepoPackage {
    RepoPackage {
        repo: repo.to_owned(),
        name: package(name),
        version: version.to_owned(),
        description: format!("About {name}"),
    }
}

fn local(name: &str, version: &str) -> LocalPackage {
    LocalPackage {
        name: package(name),
        version: version.to_owned(),
        description: format!("Installed {name}"),
    }
}

/// `linux` with headers, `linux-lts` without, `linux-zen` not installed:
/// one kernel in each state.
fn sample() -> (Vec<Kernel>, Snapshot) {
    let packages = LinuxPackages {
        sync: vec![
            repo("core", "linux", "6.14.0-4"),
            repo("core", "linux-headers", "6.14.0-4"),
            repo("core", "linux-lts", "6.12.9-1"),
            repo("core", "linux-lts-headers", "6.12.9-1"),
            repo("extra", "linux-zen", "6.13.8.zen1-1"),
            repo("extra", "linux-zen-headers", "6.13.8.zen1-1"),
        ],
        local: vec![
            local("linux", "6.14.0-4"),
            local("linux-headers", "6.14.0-4"),
            local("linux-lts", "6.12.9-1"),
        ],
    };

    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::from([
        package("linux"),
        package("linux-headers"),
        package("linux-lts"),
    ])));

    (kernels(&packages, &KernelReleases::default()), snapshot)
}

fn named<'kernels>(kernels: &'kernels [Kernel], name: &str) -> &'kernels Kernel {
    kernels
        .iter()
        .find(|kernel| kernel.name.as_str() == name)
        .unwrap()
}

/// `linux-zen` is not installed, so its running guards no removal and the
/// other refusals show.
fn zen_runs(kernels: &[Kernel]) -> Listed<'_> {
    Listed {
        kernels,
        running: Some(named(kernels, "linux-zen")),
    }
}

fn wish(changes: &ChangeSet, name: &str) -> Option<Toggle> {
    changes.wish::<Packages>(&package(name)).copied()
}

#[test]
fn kernels_are_packages_with_a_headers_pair() {
    let packages = LinuxPackages {
        sync: vec![
            repo("core", "linux", "6.14.0-4"),
            repo("core", "linux-api-headers", "6.13-1"),
            repo("core", "linux-docs", "6.14.0-4"),
            repo("core", "linux-firmware", "20250311-1"),
            repo("core", "linux-headers", "6.14.0-4"),
            repo("extra", "linux-lts", "6.12.9-1"),
            repo("extra", "linux-lts-docs", "6.12.9-1"),
            repo("extra", "linux-lts-headers", "6.12.9-1"),
            repo("extra", "linux-tools-meta", "6.14-1"),
        ],
        local: vec![local("linux", "6.14.0-4")],
    };

    let kernels = kernels(&packages, &KernelReleases::default());
    let names: Vec<&str> = kernels.iter().map(|kernel| kernel.name.as_str()).collect();

    assert_eq!(names, ["linux", "linux-lts"]);
    assert_eq!(kernels[0].headers.as_str(), "linux-headers");
    assert_eq!(kernels[0].description, "About linux");
    assert_eq!(kernels[0].repo, "core");
    assert_eq!(kernels[0].installed.as_deref(), Some("6.14.0-4"));
    assert!(!kernels[0].headers_installed);
    assert_eq!(kernels[1].installed, None);
}

#[test]
fn headers_of_another_version_or_repository_make_no_pair() {
    let packages = LinuxPackages {
        sync: vec![
            repo("core", "linux", "6.14.0-4"),
            repo("core", "linux-headers", "6.14.0-3"),
            repo("core", "linux-hardened", "6.13.9-1"),
            repo("extra", "linux-hardened-headers", "6.13.9-1"),
        ],
        local: Vec::new(),
    };

    assert_eq!(kernels(&packages, &KernelReleases::default()), []);
}

#[test]
fn a_kernel_in_several_repositories_shows_the_one_pacman_takes() {
    let packages = LinuxPackages {
        sync: vec![
            repo("chaotic-aur", "linux-cachyos", "6.14.0-1"),
            repo("chaotic-aur", "linux-cachyos-headers", "6.14.0-1"),
            repo("chaotic-aur", "linux-zen", "6.13.7.zen1-1"),
            repo("chaotic-aur", "linux-zen-headers", "6.13.7.zen1-1"),
            repo("cachyos", "linux-cachyos", "6.14.0-2"),
            repo("cachyos", "linux-cachyos-headers", "6.14.0-2"),
            repo("extra", "linux-zen", "6.13.8.zen1-1"),
            repo("extra", "linux-zen-headers", "6.13.8.zen1-1"),
            repo("garuda", "linux-zen", "6.13.8.zen1-2"),
            repo("garuda", "linux-zen-headers", "6.13.8.zen1-2"),
        ],
        local: Vec::new(),
    };

    let kernels = kernels(&packages, &KernelReleases::default());
    let shown: Vec<(&str, &str, &str)> = kernels
        .iter()
        .map(|kernel| {
            (
                kernel.name.as_str(),
                kernel.repo.as_str(),
                kernel.version.as_str(),
            )
        })
        .collect();

    assert_eq!(
        shown,
        [
            ("linux-cachyos", "chaotic-aur", "6.14.0-1"),
            ("linux-zen", "garuda", "6.13.8.zen1-2"),
        ]
    );
}

#[test]
fn installed_kernels_come_first_and_keep_their_order() {
    let (kernels, _) = sample();
    let names: Vec<&str> = kernels.iter().map(|kernel| kernel.name.as_str()).collect();

    assert_eq!(names, ["linux", "linux-lts", "linux-zen"]);

    let packages = LinuxPackages {
        sync: vec![
            repo("core", "linux", "6.14.0-4"),
            repo("core", "linux-headers", "6.14.0-4"),
            repo("extra", "linux-zen", "6.13.8.zen1-1"),
            repo("extra", "linux-zen-headers", "6.13.8.zen1-1"),
        ],
        local: vec![local("linux-zen", "6.13.8.zen1-1")],
    };

    let reordered = super::kernels(&packages, &KernelReleases::default());
    let zen_first: Vec<&str> = reordered
        .iter()
        .map(|kernel| kernel.name.as_str())
        .collect();

    assert_eq!(zen_first, ["linux-zen", "linux"]);
}

#[test]
fn an_installed_kernel_no_repository_offers_is_still_listed() {
    let packages = LinuxPackages {
        sync: vec![
            repo("core", "linux", "6.14.0-4"),
            repo("core", "linux-headers", "6.14.0-4"),
        ],
        local: vec![
            local("linux-firmware", "20250311-1"),
            local("linux-mainline", "6.15rc1-1"),
            local("linux-mainline-headers", "6.15rc1-1"),
            local("linux-tkg", "6.14.1-1"),
        ],
    };

    let releases = KernelReleases(BTreeMap::from([(
        package("linux-tkg"),
        release("6.14.1-tkg"),
    )]));

    let kernels = kernels(&packages, &releases);
    let shown: Vec<(&str, &str, bool)> = kernels
        .iter()
        .map(|kernel| {
            (
                kernel.name.as_str(),
                kernel.repo.as_str(),
                kernel.headers_installed,
            )
        })
        .collect();

    assert_eq!(
        shown,
        [
            ("linux-mainline", "", true),
            ("linux-tkg", "", false),
            ("linux", "core", false),
        ]
    );
    assert_eq!(kernels[0].description, "Installed linux-mainline");
}

#[test]
fn a_new_kernel_is_queued_with_its_headers_and_a_second_click_takes_both_back() {
    let (kernels, snapshot) = sample();
    let zen = named(&kernels, "linux-zen");
    let mut changes = ChangeSet::default();

    toggle(zen, true, zen_runs(&kernels), &mut changes, &snapshot).unwrap();
    assert_eq!(wish(&changes, "linux-zen"), Some(Toggle::On));
    assert_eq!(wish(&changes, "linux-zen-headers"), Some(Toggle::On));

    toggle(zen, false, zen_runs(&kernels), &mut changes, &snapshot).unwrap();
    assert!(changes.is_empty());
}

#[test]
fn an_installed_kernel_is_removed_with_its_headers() {
    let (kernels, snapshot) = sample();
    let linux = named(&kernels, "linux");
    let mut changes = ChangeSet::default();

    toggle(linux, false, zen_runs(&kernels), &mut changes, &snapshot).unwrap();
    assert_eq!(wish(&changes, "linux"), Some(Toggle::Off));
    assert_eq!(wish(&changes, "linux-headers"), Some(Toggle::Off));

    toggle(linux, true, zen_runs(&kernels), &mut changes, &snapshot).unwrap();
    assert!(changes.is_empty());
}

#[test]
fn an_installed_kernel_without_headers_is_removed_alone() {
    let (kernels, snapshot) = sample();
    let lts = named(&kernels, "linux-lts");
    let mut changes = ChangeSet::default();

    toggle(lts, false, zen_runs(&kernels), &mut changes, &snapshot).unwrap();
    assert_eq!(wish(&changes, "linux-lts"), Some(Toggle::Off));
    assert_eq!(wish(&changes, "linux-lts-headers"), None);
    assert_eq!(changes.len(), 1);

    toggle(lts, true, zen_runs(&kernels), &mut changes, &snapshot).unwrap();
    assert!(changes.is_empty());
}

#[test]
fn the_running_kernel_cannot_be_removed() {
    let (kernels, snapshot) = sample();
    let linux = named(&kernels, "linux");
    let mut changes = ChangeSet::default();

    assert_eq!(
        toggle(
            linux,
            false,
            Listed {
                kernels: &kernels,
                running: Some(linux)
            },
            &mut changes,
            &snapshot
        ),
        Err(Refusal::Running)
    );
    assert!(changes.is_empty());
}

#[test]
fn without_the_running_kernel_no_kernel_is_removed() {
    let (kernels, snapshot) = sample();
    let lts = named(&kernels, "linux-lts");
    let zen = named(&kernels, "linux-zen");
    let unknown = Listed {
        kernels: &kernels,
        running: None,
    };

    let mut changes = ChangeSet::default();

    assert_eq!(
        toggle(lts, false, unknown, &mut changes, &snapshot),
        Err(Refusal::RunningUnknown)
    );
    assert!(changes.is_empty());

    toggle(zen, true, unknown, &mut changes, &snapshot).unwrap();
    assert_eq!(wish(&changes, "linux-zen"), Some(Toggle::On));
}

#[test]
fn the_last_kernel_cannot_be_removed() {
    let (kernels, snapshot) = sample();
    let linux = named(&kernels, "linux");
    let lts = named(&kernels, "linux-lts");
    let zen = named(&kernels, "linux-zen");
    let mut changes = ChangeSet::default();

    toggle(zen, true, zen_runs(&kernels), &mut changes, &snapshot).unwrap();
    toggle(lts, false, zen_runs(&kernels), &mut changes, &snapshot).unwrap();

    // linux-zen is only queued: when its installation fails, nothing boots.
    assert_eq!(
        toggle(linux, false, zen_runs(&kernels), &mut changes, &snapshot),
        Err(Refusal::LastKernel)
    );
    assert_eq!(wish(&changes, "linux"), None);
}

#[test]
fn missing_headers_are_queued_for_every_installed_kernel_that_lacks_them() {
    let (kernels, snapshot) = sample();
    let mut changes = ChangeSet::default();

    assert!(!lacks_headers(named(&kernels, "linux"), &changes));
    assert!(lacks_headers(named(&kernels, "linux-lts"), &changes));
    assert!(!lacks_headers(named(&kernels, "linux-zen"), &changes));

    assert_eq!(
        install_missing_headers(&kernels, &mut changes, &snapshot),
        1
    );
    assert_eq!(wish(&changes, "linux-lts-headers"), Some(Toggle::On));
    assert_eq!(changes.len(), 1);
    assert!(!lacks_headers(named(&kernels, "linux-lts"), &changes));
    assert_eq!(
        install_missing_headers(&kernels, &mut changes, &snapshot),
        0
    );
}

#[test]
fn a_kernel_queued_for_removal_needs_no_headers() {
    let (kernels, snapshot) = sample();
    let lts = named(&kernels, "linux-lts");
    let mut changes = ChangeSet::default();

    toggle(lts, false, zen_runs(&kernels), &mut changes, &snapshot).unwrap();

    let queued = install_missing_headers(&kernels, &mut changes, &snapshot);

    assert!(!lacks_headers(lts, &changes) && queued == 0);
}
