use egui_kittest::kittest::Queryable as _;

use super::{listed, page};

#[test]
fn a_frame_with_the_same_inputs_builds_no_config_rows() {
    let mut harness = page(listed());
    let built = harness.state().page.rows.builds();

    harness.step();
    harness.step();
    assert_eq!(harness.state().page.rows.builds(), built);

    harness.get_by_label("Show the Files").click();
    harness.run();
    assert_eq!(harness.state().page.rows.builds(), built + 1);
    assert!(super::shows(&harness, "audio.conf"));

    "grub".clone_into(&mut harness.state_mut().page.search);
    harness.run();
    assert_eq!(harness.state().page.rows.builds(), built + 2);
}

#[test]
fn the_files_of_an_opened_folder_are_indented_under_it() {
    let mut harness = page(listed());
    harness.get_by_label("Show the Files").click();
    harness.run();

    // The row's own node starts at the card's edge, its title further in.
    let left = |label: &str| {
        harness
            .query_all_by_label(label)
            .map(|node| node.rect().left())
            .fold(f32::MIN, f32::max)
    };

    let (folder, file) = (left("modprobe.d"), left("audio.conf"));
    assert!(
        file > folder + 8.0,
        "audio.conf at {file}, its folder at {folder}"
    );
}
