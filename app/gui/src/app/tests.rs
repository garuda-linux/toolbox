use egui_kittest::Harness;

use super::{App, Setup};

#[test]
fn a_frame_with_the_same_queue_builds_no_queue_rows() {
    let mut harness = Harness::builder().build_eframe(|cc| App::new(cc, Setup::default()));
    harness.run();
    let built = harness.state().views.builds();

    harness.step();
    harness.step();
    assert_eq!(harness.state().views.builds(), built);

    harness.state_mut().queue.clear();
    harness.run();
    assert_eq!(harness.state().views.builds(), built + 1);
}
