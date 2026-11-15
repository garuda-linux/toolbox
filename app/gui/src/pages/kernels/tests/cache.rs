use std::sync::Arc;

use super::super::KernelsPage;
use super::super::fixtures::system;
use crate::testing::PageFixture;

/// The listing, its rows and both missing counts come from the system,
/// the queue and dkms alone.
#[test]
fn a_frame_with_the_same_system_builds_no_rows() {
    let mut harness = PageFixture::with_snapshot(system()).harness(KernelsPage::default());
    harness.run();
    let built = harness.state().page.shown.builds();

    harness.step();
    harness.step();
    assert_eq!(harness.state().page.shown.builds(), built);

    harness.state_mut().fixture.revision += 1;
    harness.run();
    assert_eq!(harness.state().page.shown.builds(), built + 1);

    harness.state_mut().fixture.snapshot = Arc::new(system());
    harness.run();
    assert_eq!(harness.state().page.shown.builds(), built + 2);
}
