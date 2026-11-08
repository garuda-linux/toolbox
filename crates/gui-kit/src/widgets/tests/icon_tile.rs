use egui::Role;

use super::super::{
    ActionRow, SectionHeading, Tone, banner, empty_state, error_row, link_row, link_tile,
};
use crate::icons;
use crate::testing::{circle_fills, filled_rect, harness, tinted_rects};
use crate::tokens::{self, Hue, set_page_hue};

fn tile_of(hue: Hue, mut draw: impl FnMut(&mut egui::Ui) + 'static) -> (Option<egui::Rect>, bool) {
    let mut harness = harness(600.0, 200.0).build_ui(move |ui| {
        set_page_hue(ui.ctx(), hue);
        draw(ui);
    });

    harness.run();
    let tile = tokens::current(&harness.ctx).tiles.of(hue);
    (
        filled_rect(&harness, tile.fill),
        !circle_fills(&harness).is_empty(),
    )
}

fn assert_square(rect: Option<egui::Rect>, what: &str) {
    let rect = rect.unwrap_or_else(|| panic!("{what} paints no tile in the page hue"));
    assert!(
        (rect.width() - rect.height()).abs() < 0.5,
        "{what}: the tile is {rect:?}, not square"
    );
}

#[test]
fn a_section_heading_sets_its_icon_on_a_tile_of_the_page_hue() {
    let (tile, _) = tile_of(Hue::Blue, |ui| {
        SectionHeading::new(icons::RUN_BUILD, "Installed", "3 kernels").show(ui);
    });

    assert_square(tile, "a section heading");
}

#[test]
fn a_quiet_action_row_takes_the_page_hue_on_a_square_not_a_disc() {
    for tone in [Tone::Neutral, Tone::Accent] {
        let (tile, disc) = tile_of(Hue::Green, move |ui| {
            ActionRow::new(icons::TOOLS, tone, "Clean the cache").show_without_actions(ui);
        });

        assert_square(tile, "an action row");
        assert!(!disc, "{tone:?}: the icon still sits on a disc");
    }
}

#[test]
fn a_warning_row_keeps_its_status_colour() {
    let mut harness = harness(600.0, 200.0).build_ui(|ui| {
        set_page_hue(ui.ctx(), Hue::Orange);
        ActionRow::new(icons::SYSTEM_REBOOT, Tone::Warning, "Restart needed")
            .show_without_actions(ui);
    });

    harness.run();
    let tokens = tokens::current(&harness.ctx);
    assert_square(
        filled_rect(&harness, tokens.status_tile(tokens.warning).fill),
        "a warning row",
    );
    assert!(filled_rect(&harness, tokens.tiles.of(Hue::Orange).fill).is_none());
}

#[test]
fn link_tiles_and_link_rows_set_their_icon_on_the_page_tile() {
    let (on_tile, disc) = tile_of(Hue::Orange, |ui| {
        _ = link_tile(ui, icons::TOOLS, "Maintenance", "Keep it clean", 300.0);
    });

    assert_square(on_tile, "a link tile");
    assert!(!disc, "a link tile still draws a disc");

    let (on_row, _) = tile_of(Hue::Orange, |ui| {
        _ = link_row(ui, icons::GLOBE, "Forum", Role::Link);
    });

    assert_square(on_row, "a link row");
}

#[test]
fn without_a_page_hue_the_tile_is_the_accent() {
    let mut harness = harness(600.0, 200.0).build_ui(|ui| {
        SectionHeading::new(icons::RUN_BUILD, "Installed", "").show(ui);
    });

    harness.run();
    let tile = tokens::current(&harness.ctx).tiles.of(Hue::Accent);
    assert_square(filled_rect(&harness, tile.fill), "a heading without a hue");
}

fn assert_status_mark<State>(harness: &egui_kittest::Harness<'_, State>, tone: Tone, what: &str) {
    let tokens = tokens::current(&harness.ctx);
    let (tile, _) = tone.status_mark(&tokens);
    let square = filled_rect(harness, tile.fill);
    assert_square(square, what);

    let square = square.unwrap_or(egui::Rect::NOTHING);
    let glyph = tinted_rects(harness, tile.ink)
        .into_iter()
        .find(|glyph| square.contains_rect(*glyph))
        .unwrap_or_else(|| panic!("{what}: no glyph in {:?} on its tile", tile.ink));

    let off = glyph.center() - square.center();
    assert!(
        off.length() < 0.5,
        "{what}: the glyph is {off:?} off the tile's middle"
    );
}

#[test]
fn every_banner_marks_its_status_with_the_status_tile() {
    for tone in [Tone::Accent, Tone::Warning, Tone::Success, Tone::Error] {
        let mut harness = harness(800.0, 200.0).build_ui(move |ui| {
            _ = banner(ui, tone, "Two kernels are installed.", &[]);
        });

        harness.run();
        assert_status_mark(&harness, tone, &format!("a {tone:?} banner"));
    }
}

#[test]
fn error_rows_empty_states_and_status_rows_use_the_same_tile() {
    let mut row = harness_of(|ui| {
        _ = error_row(ui, "The list did not load.", "Retry");
    });

    row.run();
    assert_status_mark(&row, Tone::Error, "an error row");

    let mut empty = harness_of(|ui| {
        _ = empty_state(ui, icons::DIALOG_ERROR, "Nothing was read", "");
    });

    empty.run();
    assert_status_mark(&empty, Tone::Error, "an error empty state");

    let mut status = harness_of(|ui| {
        ActionRow::new(icons::DIALOG_WARNING, Tone::Warning, "2 warnings").show_without_actions(ui);
    });

    status.run();
    assert_status_mark(&status, Tone::Warning, "a status row");
}

fn harness_of(draw: impl FnMut(&mut egui::Ui) + 'static) -> egui_kittest::Harness<'static> {
    harness(800.0, 300.0).build_ui(draw)
}
