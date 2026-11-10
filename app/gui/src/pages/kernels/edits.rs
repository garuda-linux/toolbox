#[cfg(test)]
mod tests;

use toolbox_core::domains::{DkmsBuilds, Packages};
use toolbox_core::features::kernels::{Kernel, lacks_headers};
use toolbox_core::{ChangeSet, Queued, Snapshot, Toggle};
use toolbox_plan::DkmsBuild;

/// Queues the headers of an installed kernel that lacks them, or takes
/// the queued ones back. Returns whether the queue changed.
pub fn set_headers(
    kernel: &Kernel,
    wish: Toggle,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> bool {
    let on = wish.is_on();
    let queued = changes.wish::<Packages>(&kernel.headers) == Some(&Toggle::On);

    match (on, queued) {
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

pub fn set_builds(
    builds: &[DkmsBuild],
    wish: Toggle,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) {
    let on = wish.is_on();
    for build in builds {
        if on {
            changes.want::<DkmsBuilds>(build.clone(), Queued, snapshot);
        } else {
            changes.remove::<DkmsBuilds>(build);
        }
    }
}
