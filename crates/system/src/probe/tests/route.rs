//! The helper's answer for the user's build folder is a part of every probed
//! snapshot, so the split path can read it before it queues a build.

use toolbox_state::aur::AurRoute;

use super::fakes::{FakePackages, sources};
use super::{host, installed, probe_snapshot};

#[test]
fn the_helper_route_is_a_part_of_every_probed_snapshot() {
    let (snapshot, _) = probe_snapshot(&sources(FakePackages::new(Ok(installed())), host()));

    assert_eq!(snapshot.part::<AurRoute>(), Some(&AurRoute::Accepted));
}
