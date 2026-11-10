//! The headers a kernel needs, queued for the kernels that lack them.

use super::super::{install_missing_headers, lacks_headers, toggle};
use super::{named, sample, wish, zen_runs};
use crate::{ChangeSet, Toggle};

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

    toggle(
        lts,
        Toggle::Off,
        zen_runs(&kernels),
        &mut changes,
        &snapshot,
    )
    .unwrap();

    let queued = install_missing_headers(&kernels, &mut changes, &snapshot);

    assert!(!lacks_headers(lts, &changes) && queued == 0);
}
