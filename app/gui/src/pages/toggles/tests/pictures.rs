use toolbox_catalog::ToggleCatalog;
use toolbox_gui_kit::testing::{filled_rect, filled_rects, painted_text, textured_rects};
use toolbox_gui_kit::tokens::{self, LETTER_HUES};
use toolbox_gui_kit::widgets::{dim_opacity, hue_of};

use super::super::entry::tile_package;
use super::page;
use crate::catalogs::CatalogId;
use crate::pages::rows::PICTURE_SIZE;

#[test]
fn the_tile_hue_follows_the_package_an_entry_checks_else_its_id() {
    let catalog = ToggleCatalog::parse(
        concat!(
            "[[section]]\nid = \"audio\"\ntitle = \"Audio\"\nicon = \"x\"\n",
            "[[section.entry]]\nid = \"strawberry-player\"\ntitle = \"Strawberry Player\"\n",
            "description = \"D\"\ncheck = { package = \"strawberry\" }\n",
            "[[section.entry]]\nid = \"bluetooth\"\ntitle = \"Bluetooth\"\n",
            "description = \"D\"\ncheck = { service = \"bluetooth.service\" }\n",
        ),
        "test.toml",
    )
    .expect("a valid catalog");

    let packages: Vec<&str> = catalog.entries().map(tile_package).collect();
    assert_eq!(packages, ["strawberry", "bluetooth"]);
}

#[test]
fn an_entry_without_a_picture_shows_its_initial_on_its_package_hue() {
    let harness = page(CatalogId::Setup, "strawberry");
    let tile = tokens::current(&harness.ctx)
        .tiles
        .letters()
        .of(hue_of("strawberry"));

    let at = filled_rect(&harness, tile.fill).expect("no letter tile in the package's hue");
    assert!(
        (at.width() - PICTURE_SIZE).abs() < 0.5,
        "the tile is {at:?}"
    );
    assert!(
        (at.height() - PICTURE_SIZE).abs() < 0.5,
        "the tile is {at:?}"
    );
    assert!(
        painted_text(&harness, "S").is_some(),
        "no initial on the tile"
    );
}

#[test]
fn rows_with_and_without_a_picture_start_their_picture_at_one_edge() {
    let harness = page(CatalogId::Gaming, "");
    let fills = tokens::current(&harness.ctx).tiles.letters();
    let mut slots: Vec<egui::Rect> = (0..LETTER_HUES)
        .flat_map(|hue| filled_rects(&harness, fills.of(hue).fill))
        .chain(textured_rects(&harness))
        .filter(|rect| (rect.width() - PICTURE_SIZE).abs() < 0.5)
        .collect();

    slots.sort_by(|one, other| one.left().total_cmp(&other.left()));
    let (Some(first), Some(last)) = (slots.first(), slots.last()) else {
        panic!("the page shows no pictures or letter tiles");
    };

    assert!(slots.len() > 1, "only one row shows a picture");
    assert!(
        (last.left() - first.left()).abs() < 0.5,
        "pictures start at {} and {}",
        first.left(),
        last.left()
    );
}

#[test]
fn the_tile_of_an_entry_that_waits_for_another_fades_as_its_title() {
    // JamesDSP needs PipeWire, which the empty system lacks.
    let harness = page(CatalogId::Components, "jamesdsp");
    let tokens = tokens::current(&harness.ctx);
    let tile = tokens.tiles.letters().of(hue_of("jamesdsp"));
    let faded = tile.fill.gamma_multiply(dim_opacity(&tokens));

    assert!(
        filled_rect(&harness, faded).is_some(),
        "no faded letter tile"
    );
    assert!(
        filled_rect(&harness, tile.fill).is_none(),
        "the tile keeps its full fill"
    );
}
