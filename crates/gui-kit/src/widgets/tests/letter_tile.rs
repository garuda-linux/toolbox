use egui::{Rect, pos2, vec2};

use super::super::dim::{dim_opacity, dim_picture};
use super::super::letter_tile::{hue_of, initial, paint_letter_tile};
use crate::fonts::Weight;
use crate::testing::{filled_rect, harness, painted_text};
use crate::tokens::letters::LETTER_HUES;
use crate::tokens::{self};

#[test]
fn the_initial_is_the_first_letter_of_the_shown_name_upper_case() {
    assert_eq!(initial("Strawberry Player"), Some('S'));
    assert_eq!(initial("firefox"), Some('F'));
}

#[test]
fn signs_and_spaces_before_the_name_are_skipped() {
    assert_eq!(initial("  _lib32-mesa"), Some('L'));
    assert_eq!(initial("[beta] app"), Some('B'));
}

#[test]
fn a_name_starting_with_a_digit_keeps_the_digit() {
    assert_eq!(initial("0ad"), Some('0'));
    assert_eq!(initial("7-zip"), Some('7'));
}

#[test]
fn letters_beyond_ascii_are_upper_cased_or_kept() {
    assert_eq!(initial("élan"), Some('É'));
    assert_eq!(initial("中文输入法"), Some('中'));

    assert_eq!(initial("ßtest"), Some('S'));
}

#[test]
fn a_name_without_a_letter_or_digit_has_no_initial() {
    assert_eq!(initial(""), None);
    assert_eq!(initial("+-*"), None);
}

#[test]
fn the_hue_stays_the_same_for_a_package_across_runs() {
    // Pinned, as a hash that changes with the toolchain would recolour
    // every tile after an update.
    assert_eq!(hue_of("firefox"), 6);
    assert_eq!(hue_of("steam"), 1);
    assert_eq!(hue_of("gimp"), 7);
    assert_eq!(hue_of("obs-studio"), 2);
}

#[test]
fn package_names_spread_over_every_hue() {
    let names = [
        "firefox",
        "strawberry",
        "steam",
        "linux-zen",
        "gimp",
        "obs-studio",
        "neovim",
        "lutris",
        "vlc",
        "kdenlive",
        "wine",
        "0ad",
        "thunderbird",
        "inkscape",
        "krita",
        "blender",
        "audacity",
        "discord",
        "telegram-desktop",
        "libreoffice-fresh",
    ];

    let mut seen = [false; LETTER_HUES];
    for name in names {
        if let Some(hit) = seen.get_mut(hue_of(name)) {
            *hit = true;
        }
    }

    assert!(
        seen.iter().all(|hit| *hit),
        "hues no name reached: {seen:?}"
    );
}

#[test]
fn the_tile_fills_the_slot_in_the_package_hue_with_the_initial_on_it() {
    let at = Rect::from_min_size(pos2(20.0, 20.0), vec2(32.0, 32.0));
    let mut harness = harness(200.0, 100.0).build_ui(move |ui| {
        paint_letter_tile(ui, at, 6, "Strawberry Player", "strawberry");
    });

    harness.run();

    let tile = tokens::current(&harness.ctx)
        .tiles
        .letters()
        .of(hue_of("strawberry"));

    assert_eq!(filled_rect(&harness, tile.fill), Some(at));

    let letter = painted_text(&harness, "S").expect("the tile paints no initial");
    let format = &letter
        .galley
        .job
        .sections
        .first()
        .expect("the initial has no text section")
        .format;

    assert_eq!(format.color, tile.ink);
    assert_eq!(format.font_id.family, Weight::SemiBold.family());

    let inked = letter.galley.mesh_bounds.translate(letter.pos.to_vec2());
    assert!(at.contains_rect(inked), "the initial leaves its tile");
    assert!(
        (inked.center() - at.center()).length() < 1.5,
        "the initial sits off centre: {inked:?} in {at:?}"
    );
    assert!(
        inked.height() > at.height() * 0.3,
        "the initial is too small for its tile: {inked:?}"
    );
}

#[test]
fn a_name_without_an_initial_takes_the_package_names() {
    let at = Rect::from_min_size(pos2(20.0, 20.0), vec2(32.0, 32.0));
    let mut harness = harness(200.0, 100.0).build_ui(move |ui| {
        paint_letter_tile(ui, at, 6, "++", "gimp");
    });

    harness.run();

    assert!(painted_text(&harness, "G").is_some());
}

#[test]
fn a_picture_beside_dim_text_fades_as_far_as_the_text() {
    let ctx = egui::Context::default();
    let tokens = tokens::current(&ctx);
    let share = dim_opacity(&tokens);
    let faded = tokens::mix(tokens.card, tokens.text, share);

    for (one, other) in faded.to_array().iter().zip(tokens.dim.to_array()) {
        assert!(
            one.abs_diff(other) <= 8,
            "text faded to {share} is {faded:?}, the dim text {:?}",
            tokens.dim
        );
    }
}

#[test]
fn the_tile_of_a_row_that_cannot_change_takes_the_dim_texts_fade() {
    let at = Rect::from_min_size(pos2(20.0, 20.0), vec2(32.0, 32.0));
    let mut harness = harness(200.0, 100.0).build_ui(move |ui| {
        dim_picture(ui, at, true, |ui| {
            paint_letter_tile(ui, at, 6, "Strawberry Player", "strawberry");
        });
    });

    harness.run();

    let tokens = tokens::current(&harness.ctx);
    let tile = tokens.tiles.letters().of(hue_of("strawberry"));
    let faded = tile.fill.gamma_multiply(dim_opacity(&tokens));

    assert_eq!(filled_rect(&harness, faded), Some(at));
    assert_eq!(filled_rect(&harness, tile.fill), None);
}

#[test]
fn the_tile_of_a_row_that_can_change_keeps_its_fill() {
    let at = Rect::from_min_size(pos2(20.0, 20.0), vec2(32.0, 32.0));
    let mut harness = harness(200.0, 100.0).build_ui(move |ui| {
        dim_picture(ui, at, false, |ui| {
            paint_letter_tile(ui, at, 6, "Strawberry Player", "strawberry");
        });
    });

    harness.run();

    let tile = tokens::current(&harness.ctx)
        .tiles
        .letters()
        .of(hue_of("strawberry"));

    assert_eq!(filled_rect(&harness, tile.fill), Some(at));
}
