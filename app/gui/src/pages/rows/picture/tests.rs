use std::path::Path;

use egui::{Rect, pos2, vec2};
use toolbox_gui_kit::testing::{filled_rect, harness, painted_text, textured_rects};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::hue_of;
use toolbox_plan::PackageName;

use super::{Initial, package_picture, picture_or_initial};
use crate::pages::rows::PICTURE_SIZE;

fn slot() -> Rect {
    Rect::from_min_size(pos2(10.0, 10.0), vec2(PICTURE_SIZE, PICTURE_SIZE))
}

fn initial<'name>(shown: &'name str, package: &'name str) -> Initial<'name> {
    Initial { shown, package }
}

#[test]
fn a_package_without_a_finder_shows_the_initial_of_its_shown_name() {
    let package = PackageName::new("strawberry").expect("a valid name");
    let mut harness = harness(100.0, 60.0).build_ui(move |ui| {
        let tile = initial("Strawberry Player", package.as_str());
        package_picture(ui, None, Some(&package), tile, slot());
    });

    harness.run();
    let tile = tokens::current(&harness.ctx)
        .tiles
        .letters()
        .of(hue_of("strawberry"));

    assert_eq!(filled_rect(&harness, tile.fill), Some(slot()));
    assert!(painted_text(&harness, "S").is_some());
}

#[test]
fn a_picture_that_cannot_be_read_shows_the_initial() {
    let mut harness = harness(100.0, 60.0).build_ui(|ui| {
        let tile = initial("Missing", "missing");
        _ = picture_or_initial(ui, "file:///nowhere/missing.png", slot(), tile);
    });

    harness.run();
    assert!(painted_text(&harness, "M").is_some());
    assert!(textured_rects(&harness).is_empty(), "an image was painted");
}

#[test]
fn a_loaded_picture_replaces_the_initial() {
    let file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog/icons/gaming/steam.png");

    let uri = format!("file://{}", file.display());
    let mut harness = harness(100.0, 60.0).build_ui(move |ui| {
        _ = picture_or_initial(ui, &uri, slot(), initial("Steam", "steam"));
    });

    // The file is read and decoded on threads of egui's loaders.
    for _ in 0..200_u32 {
        harness.step();
        if !textured_rects(&harness).is_empty() {
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    assert_eq!(textured_rects(&harness), [slot()]);
    assert!(
        painted_text(&harness, "S").is_none(),
        "the initial still shows"
    );
}
