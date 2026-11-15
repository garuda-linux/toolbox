//! A code block: its copy button sits at the block's right edge, however
//! short the code is.

use egui_kittest::kittest::Queryable as _;

use super::page_on;

#[test]
fn every_copy_button_sits_at_the_right_edge() {
    let mut harness = page_on("cli.md");
    harness.set_size(egui::vec2(1024.0, 4000.0));
    harness.run();

    let rights: Vec<f32> = harness
        .query_all_by_label("Copy code")
        .map(|node| node.rect().right())
        .collect();

    assert!(rights.len() > 1, "cli.md shows fewer than two code blocks");
    let widest = rights.iter().copied().fold(f32::MIN, f32::max);
    for right in &rights {
        assert!(
            (widest - right).abs() < 1.0,
            "a copy button ends at {right}, the rightmost at {widest}"
        );
    }
}
