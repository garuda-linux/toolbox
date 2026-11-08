use toolbox_core::domains::{DkmsBuilds, Packages};
use toolbox_core::{ChangeSet, Toggle};

use super::super::fixtures::{dkms, package, system, vboxhost_build};
use super::super::view::Listing;
use super::{set_builds, set_headers};

#[test]
fn headers_queue_for_a_kernel_that_lacks_them_and_go_back() {
    let snapshot = system();
    let listing = Listing::of(&snapshot).unwrap();
    let lts = listing.kernel(&package("linux-lts")).unwrap();
    let mut changes = ChangeSet::default();

    assert!(set_headers(lts, true, &mut changes, &snapshot));
    assert_eq!(
        changes.wish::<Packages>(&package("linux-lts-headers")),
        Some(&Toggle::On)
    );

    assert!(set_headers(lts, false, &mut changes, &snapshot));
    assert!(changes.is_empty());
}

#[test]
fn a_kernel_with_headers_queues_none() {
    let snapshot = system();
    let listing = Listing::of(&snapshot).unwrap();
    let linux = listing.kernel(&package("linux")).unwrap();
    let zen = listing.kernel(&package("linux-zen")).unwrap();
    let mut changes = ChangeSet::default();

    assert!(!set_headers(linux, true, &mut changes, &snapshot));
    assert!(
        !set_headers(zen, true, &mut changes, &snapshot),
        "not installed"
    );
    assert!(changes.is_empty());
}

#[test]
fn builds_queue_and_go_back() {
    let snapshot = system();
    let listing = Listing::of(&snapshot).unwrap();
    let builds = listing.all_builds(&dkms());
    let mut changes = ChangeSet::default();

    set_builds(&builds, true, &mut changes, &snapshot);
    assert_eq!(builds, [vboxhost_build()]);
    assert!(changes.wish::<DkmsBuilds>(&vboxhost_build()).is_some());

    set_builds(&builds, false, &mut changes, &snapshot);
    assert!(changes.is_empty());
}
