use toolbox_core::Toggle;
use toolbox_core::domains::ModuleBlacklist;
use toolbox_core::testing::Queue as _;

use super::{module, page, tab};

#[test]
fn a_frame_with_the_same_inputs_builds_no_driver_rows() {
    let mut harness = page();
    tab(&mut harness, "Drivers");
    let built = harness.state().page.driver_rows.builds();

    harness.step();
    harness.step();
    assert_eq!(harness.state().page.driver_rows.builds(), built);

    "e1000".clone_into(&mut harness.state_mut().page.drivers.search);
    harness.run();
    assert_eq!(harness.state().page.driver_rows.builds(), built + 1);

    let state = harness.state_mut();
    let snapshot = std::sync::Arc::clone(&state.fixture.snapshot);
    state
        .fixture
        .changes
        .want::<ModuleBlacklist>(module("e1000e"), Toggle::On, &snapshot);
    state.fixture.revision += 1;
    harness.run();
    assert_eq!(harness.state().page.driver_rows.builds(), built + 2);
}
