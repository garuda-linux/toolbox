use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::domains::{AurPackages, AurWish};
use toolbox_core::testing::Queue as _;
use toolbox_core::{ChangeSet, Snapshot};

use super::queued;
use crate::common::launch;

/// A queued AUR build has no build order yet, so Apply holds and says why.
#[test]
fn apply_waits_for_the_build_order_of_a_queued_aur_build() {
    let mut changes = ChangeSet::default();
    changes.want::<AurPackages>(
        "yay-bin".parse().unwrap(),
        AurWish::Waiting,
        &Snapshot::default(),
    );
    let harness = launch(queued(changes));

    let apply = harness.get_by_label("Apply");
    assert!(apply.accesskit_node().is_disabled());
    assert_eq!(
        apply.accesskit_node().description().as_deref(),
        Some(
            "Apply waits for the build orders of the queued AUR packages. Take those packages out of the queue to apply the rest."
        )
    );
}
