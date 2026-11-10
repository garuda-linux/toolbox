//! The Kernels page's other clicks: a kernel's headers and the DKMS
//! builds it lacks.

use toolbox_plan::{DkmsBuild, KernelRelease};

use super::{DkmsEntry, Kernel, KernelReleases, builds, lacks_headers};
use crate::domains::{DkmsBuilds, Packages};
use crate::{ChangeSet, Queued, Snapshot, Toggle};

/// The release dkms builds for: only an installed kernel with its
/// headers, since dkms needs both.
pub fn build_release<'releases>(
    kernel: &Kernel,
    releases: &'releases KernelReleases,
) -> Option<&'releases KernelRelease> {
    (kernel.is_installed() && kernel.headers_installed)
        .then(|| releases.0.get(&kernel.name))
        .flatten()
}

/// The builds dkms lacks for `kernel`, none for one [`build_release`]
/// rules out.
pub fn builds_for(
    kernel: &Kernel,
    releases: &KernelReleases,
    entries: &[DkmsEntry],
) -> Vec<DkmsBuild> {
    build_release(kernel, releases)
        .map(|release| builds(release, entries))
        .unwrap_or_default()
}

/// Queues the headers of an installed kernel that lacks them, or takes
/// the queued ones back. Returns whether the queue changed.
pub fn set_headers(
    kernel: &Kernel,
    wish: Toggle,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> bool {
    let queued = changes.wish::<Packages>(&kernel.headers) == Some(&Toggle::On);

    match (wish.is_on(), queued) {
        (true, false) if lacks_headers(kernel, changes) => {
            changes.want::<Packages>(kernel.headers.clone(), Toggle::On, snapshot);
            true
        }
        (false, true) => {
            changes.remove::<Packages>(&kernel.headers);
            true
        }
        (true | false, _) => false,
    }
}

/// Queues `builds`, or takes them back.
pub fn set_builds(
    builds: &[DkmsBuild],
    wish: Toggle,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) {
    for build in builds {
        if wish.is_on() {
            changes.want::<DkmsBuilds>(build.clone(), Queued, snapshot);
        } else {
            changes.remove::<DkmsBuilds>(build);
        }
    }
}
