use egui_kittest::kittest::Queryable as _;

use super::{listed, page};

#[test]
fn a_frame_with_the_same_inputs_builds_no_config_rows() {
    let mut harness = page(listed());
    let built = harness.state().page.rows.builds();

    harness.step();
    harness.step();
    assert_eq!(harness.state().page.rows.builds(), built);

    harness.get_by_label("Show the files").click();
    harness.run();
    assert_eq!(harness.state().page.rows.builds(), built + 1);
    assert!(super::shows(&harness, "audio.conf"));

    "grub".clone_into(&mut harness.state_mut().page.search);
    harness.run();
    assert_eq!(harness.state().page.rows.builds(), built + 2);
}
