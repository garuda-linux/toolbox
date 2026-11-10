//! A kernel's headers and its DKMS builds, queued and taken back.

use toolbox_plan::{DkmsBuild, DkmsModuleName, DkmsVersion};
use toolbox_state::kernels::parse_status;

use super::super::{KernelReleases, builds_for, set_builds, set_headers};
use super::{named, release, sample, wish};
use crate::domains::DkmsBuilds;
use crate::test_support::package;
use crate::{ChangeSet, Toggle};

/// v4l2loopback is added but built for no kernel, broadcom-wl is built
/// for `7.3.1-1-garuda`.
const DKMS_STATUS: &str = toolbox_state::testing::kernels::dkms::ADDED_STDOUT;

/// `linux` runs the release broadcom-wl is built for, `linux-lts` the
/// other one of the capture.
fn releases() -> KernelReleases {
    KernelReleases(
        [("linux", "7.3.1-1-garuda"), ("linux-lts", "6.18.60-1-lts")]
            .into_iter()
            .map(|(kernel, name)| (package(kernel), release(name)))
            .collect(),
    )
}

fn v4l2loopback() -> DkmsBuild {
    DkmsBuild {
        module: DkmsModuleName::new("v4l2loopback").unwrap(),
        version: DkmsVersion::new("0.15.4").unwrap(),
        kernel: release("7.3.1-1-garuda"),
    }
}

#[test]
fn headers_queue_for_a_kernel_that_lacks_them_and_go_back() {
    let (kernels, snapshot) = sample();
    let lts = named(&kernels, "linux-lts");
    let mut changes = ChangeSet::default();

    assert!(set_headers(lts, Toggle::On, &mut changes, &snapshot));
    assert_eq!(wish(&changes, "linux-lts-headers"), Some(Toggle::On));

    assert!(set_headers(lts, Toggle::Off, &mut changes, &snapshot));
    assert!(changes.is_empty());
}

#[test]
fn a_kernel_with_headers_or_not_installed_queues_none() {
    let (kernels, snapshot) = sample();
    let mut changes = ChangeSet::default();

    assert!(!set_headers(
        named(&kernels, "linux"),
        Toggle::On,
        &mut changes,
        &snapshot
    ));
    assert!(!set_headers(
        named(&kernels, "linux-zen"),
        Toggle::On,
        &mut changes,
        &snapshot
    ));
    assert!(changes.is_empty());
}

/// dkms builds only for an installed kernel with its headers.
#[test]
fn only_a_kernel_with_headers_lacks_builds() {
    let (kernels, _) = sample();
    let entries = parse_status(DKMS_STATUS);

    assert_eq!(
        builds_for(named(&kernels, "linux"), &releases(), &entries),
        [v4l2loopback()]
    );
    assert_eq!(
        builds_for(named(&kernels, "linux-lts"), &releases(), &entries),
        []
    );
    assert_eq!(
        builds_for(named(&kernels, "linux-zen"), &releases(), &entries),
        []
    );
}

#[test]
fn builds_queue_and_go_back() {
    let (_, snapshot) = sample();
    let builds = [v4l2loopback()];
    let mut changes = ChangeSet::default();

    set_builds(&builds, Toggle::On, &mut changes, &snapshot);
    assert!(changes.wish::<DkmsBuilds>(&v4l2loopback()).is_some());

    set_builds(&builds, Toggle::Off, &mut changes, &snapshot);
    assert!(changes.is_empty());
}
