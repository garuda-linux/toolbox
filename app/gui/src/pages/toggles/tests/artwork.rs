use toolbox_gui_kit::testing::{painted_text, textured_rects};

use super::page;
use super::tops::LETTERS;
use crate::catalogs::CatalogId;
use crate::pages::rows::PICTURE_SIZE;
use crate::testing::{drawing_top, letters_top};

#[test]
fn a_bundled_picture_starts_at_the_titles_letters() {
    for (search, title) in [("bottles", "Bottles"), ("heroic", "Heroic Games Launcher")] {
        let mut harness = page(CatalogId::Gaming, search);
        let slot = textured_rects(&harness)
            .into_iter()
            .find(|rect| (rect.width() - PICTURE_SIZE).abs() < 0.5)
            .unwrap_or_else(|| panic!("{search}: no picture is painted"));

        let shape = painted_text(&harness, title).expect("the title is painted");
        let text = shape.galley.rect.translate(shape.pos.to_vec2());
        let letters = letters_top(&mut harness, text).expect("the title shows light letters");
        let artwork = drawing_top(&mut harness, slot).expect("the picture is painted");

        assert!(
            (artwork - letters).abs() < LETTERS,
            "{search}: the picture's artwork starts at {artwork}, the title's letters at {letters}"
        );
    }
}
