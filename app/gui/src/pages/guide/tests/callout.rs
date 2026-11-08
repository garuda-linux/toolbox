use toolbox_gui_kit::testing::{filled_rect, painted_text};
use toolbox_gui_kit::tokens;

use super::page_on;
use crate::testing::letters_top;

const LETTERS: f32 = 1.5;

/// Notes and warnings draw their mark the same way. Boot repair's
/// warning is the one near enough the top of its page to show.
#[test]
fn a_callouts_status_tile_starts_at_its_labels_letters() {
    let mut harness = page_on("boot-repair.md");
    let tokens = tokens::current(&harness.ctx);
    let tile = filled_rect(&harness, tokens.status_tile(tokens.warning).fill)
        .expect("the warning's status tile is painted");

    let shape = painted_text(&harness, "Warning").expect("the label is painted");
    let text = shape.galley.rect.translate(shape.pos.to_vec2());
    let letters = letters_top(&mut harness, text).expect("the title shows light letters");

    assert!(
        (tile.top() - letters).abs() < LETTERS,
        "the tile starts at {}, the label's letters at {letters}",
        tile.top()
    );
}
