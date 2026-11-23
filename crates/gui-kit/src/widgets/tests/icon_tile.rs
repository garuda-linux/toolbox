//! Icons draw plain, as KDE draws them: no tile behind a row, a link or
//! a status, and the sizes of Kirigami's icons.

use egui::{Color32, Rect, pos2};

use super::super::icon_tile::TileSize;
use super::super::{
    ActionRow, NavLook, SectionHeading, Tone, empty_state, link_tile, nav_row, paint_status_tile,
};
use crate::icons::{self, Icon, colour};
use crate::testing::{circle_fills, filled_rects, harness, textured_rects};
use crate::tokens::{self, Hue, Sizes, Tokens, set_page_hue};

/// Every fill a tile of any hue or status could paint.
fn tile_fills(tokens: &Tokens) -> Vec<Color32> {
    let mut fills: Vec<Color32> = [Hue::Blue, Hue::Green, Hue::Orange]
        .into_iter()
        .map(|hue| tokens.tiles.of(hue).fill)
        .collect();

    fills.extend(
        [
            tokens.accent,
            tokens.secondary,
            tokens.warning,
            tokens.success,
            tokens.error,
        ]
        .map(|colour| tokens.status_tile(colour).fill),
    );
    fills
}

fn tiles_painted<State>(harness: &egui_kittest::Harness<'_, State>) -> Vec<Rect> {
    let tokens = tokens::current(&harness.ctx);
    tile_fills(&tokens)
        .into_iter()
        .flat_map(|fill| filled_rects(harness, fill))
        .collect()
}

fn status_icon(tone: Tone) -> Icon {
    let tokens = Tokens::derive(
        &crate::Palette::fallback(egui::Theme::Dark),
        egui::Theme::Dark,
    );

    tone.status_mark(&tokens).1
}

#[test]
fn a_section_heading_draws_no_icon() {
    let mut harness = harness(600.0, 200.0).build_ui(|ui| {
        SectionHeading::new("Installed", "3 kernels").show(ui);
    });

    harness.run();
    assert_eq!(tiles_painted(&harness), Vec::<Rect>::new());
    assert_eq!(circle_fills(&harness), Vec::<Color32>::new());
}

#[test]
fn a_row_icon_is_a_plain_colour_icon_on_no_tile() {
    let mut harness = harness(600.0, 200.0).build_ui(|ui| {
        set_page_hue(ui.ctx(), Hue::Green);
        ActionRow::new(icons::NETWORK_WIRED, "Wired").show_without_actions(ui);
    });

    harness.run();
    assert_eq!(tiles_painted(&harness), Vec::<Rect>::new());
}

#[test]
fn a_row_glyph_takes_its_colour_icon_and_a_colourless_one_keeps_its_glyph() {
    assert_eq!(icons::NETWORK_WIRED.for_row(), colour::NETWORK_WIRED);
    assert_eq!(icons::CHECKMARK.for_row(), icons::CHECKMARK);
}

#[test]
fn icon_sizes_follow_kirigami() {
    assert_eq!(TileSize::Row.edge(), 22.0);
    assert_eq!(TileSize::Line.edge(), 22.0);
}

#[test]
fn a_link_tile_paints_its_icon_without_a_tile() {
    let mut harness = harness(600.0, 200.0).build_ui(|ui| {
        set_page_hue(ui.ctx(), Hue::Orange);
        link_tile(
            ui,
            icons::NETWORK_WIRED,
            "Network",
            "Wired and wireless",
            360.0,
        );
    });

    harness.run();
    assert_eq!(tiles_painted(&harness), Vec::<Rect>::new());
}

#[test]
fn the_statuses_take_dialog_icons_and_neutral_takes_information() {
    assert_eq!(status_icon(Tone::Accent).name(), "dialog-information");
    assert_eq!(status_icon(Tone::Neutral).name(), "dialog-information");
    assert_eq!(status_icon(Tone::Warning).name(), "dialog-warning");
    assert_eq!(status_icon(Tone::Success).name(), "dialog-positive");
    assert_eq!(status_icon(Tone::Error).name(), "dialog-error");
    assert!(!status_icon(Tone::Error).is_glyph());
}

#[test]
fn status_marks_draw_their_icons_without_a_tile() {
    let mut harness = harness(300.0, 200.0).build_ui(|ui| {
        paint_status_tile(ui, pos2(40.0, 40.0), TileSize::Line, Tone::Warning);
        paint_status_tile(ui, pos2(80.0, 40.0), TileSize::Line, Tone::Error);
    });

    harness.run();
    assert_eq!(tiles_painted(&harness), Vec::<Rect>::new());
}

#[test]
fn a_sidebar_entry_is_item_high_and_its_icon_is_22_points() {
    let look = NavLook {
        expanded: true,
        height: Sizes::DEFAULT.nav_row_height,
    };

    let mut harness = harness(300.0, 200.0).build_ui(move |ui| {
        let entry = nav_row(ui, icons::GO_HOME, "Home", true, look);
        assert!((entry.rect.height() - 36.0).abs() < 0.5, "{entry:?}");
    });

    harness.run();
    let icon = textured_rects(&harness).first().copied();
    assert!(
        icon.is_some_and(|rect| (rect.width() - 22.0).abs() < 0.5),
        "{icon:?}"
    );
}

#[test]
fn an_empty_state_draws_its_icon_without_a_tile() {
    let mut harness = harness(400.0, 400.0).build_ui(|ui| {
        empty_state(ui, icons::HELP_ABOUT, "Nothing here", "Try again later.");
    });

    harness.run();
    assert_eq!(tiles_painted(&harness), Vec::<Rect>::new());
}
