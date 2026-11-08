use egui_kittest::kittest::Queryable as _;
use toolbox_gui_kit::testing::painted_text;

use super::{listed, page};
use crate::pages::Refusals;

const OUTSIDE: &str = "Not edited here: the link leads out of the system's config folders, to a file of a package or a generated one";

#[test]
fn why_a_file_is_not_edited_wraps_instead_of_being_cut() {
    let mut harness = page(listed(), Refusals::default());
    harness.set_size(egui::vec2(700.0, 900.0));
    harness.run();
    harness.run();

    let status = harness.get_by_label(OUTSIDE).rect();
    let line = harness
        .ctx
        .fonts_mut(|fonts| fonts.row_height(&egui::FontId::proportional(14.0)));

    assert!(
        status.height() > line * 1.5,
        "the reason stays on one cut line: {status:?}"
    );

    assert!(
        status.right() <= 700.0,
        "the reason runs past the window: {status:?}"
    );
}

#[test]
fn a_folder_counts_its_files_as_plain_text_without_a_state_dot() {
    let harness = page(listed(), Refusals::default());

    // The row is one node for screen readers, so the count is found by
    // what is painted.
    let text = painted_text(&harness, "2 files").expect("the folder's file count");
    let count = text.galley.rect.translate(text.pos.to_vec2());

    let dotted = harness.output().shapes.iter().any(|clipped| {
        matches!(&clipped.shape, egui::Shape::Circle(circle)
            if count.expand(8.0).contains(circle.center))
    });

    assert!(!dotted, "the count at {count:?} has a state dot");
}
