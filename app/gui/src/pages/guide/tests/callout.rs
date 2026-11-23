use toolbox_gui_kit::testing::{painted_text, textured_rects};

use super::page_on;
use crate::testing::letters_top;

const LETTERS: f32 = 1.5;

/// Notes and warnings draw their mark the same way. Boot repair's
/// warning is the one near enough the top of its page to show.
#[test]
fn a_callouts_status_tile_starts_at_its_labels_letters() {
    let mut harness = page_on("boot-repair.md");

    let shape = painted_text(&harness, "Warning").expect("the label is painted");
    let text = shape.galley.rect.translate(shape.pos.to_vec2());
    let letters = letters_top(&mut harness, text).expect("the title shows light letters");

    // The mark is the colour status icon, a textured image, drawn left of
    // its label.
    let tile = textured_rects(&harness)
        .into_iter()
        .filter(|rect| rect.right() <= text.left() + 0.5)
        .max_by(|one, other| one.right().total_cmp(&other.right()))
        .expect("the warning's status mark is painted");

    assert!(
        (tile.top() - letters).abs() < LETTERS,
        "the tile starts at {}, the label's letters at {letters}",
        tile.top()
    );
}
