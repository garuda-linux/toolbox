#![expect(
    clippy::disallowed_methods,
    reason = "tests build desktop palettes from literal colours"
)]

use egui::{Color32, Theme};

use super::super::tiles::{Hue, page_hue, set_page_hue};
use super::super::{Tokens, contrast};
use crate::palette::{Palette, TileHues};

const HUES: [Hue; 4] = [Hue::Accent, Hue::Blue, Hue::Green, Hue::Orange];

fn both() -> [(Theme, Tokens); 2] {
    [Theme::Dark, Theme::Light]
        .map(|theme| (theme, Tokens::derive(&Palette::fallback(theme), theme)))
}

#[test]
fn a_glyph_reads_at_three_to_one_on_its_tile() {
    for (theme, tokens) in both() {
        for hue in HUES {
            let tile = tokens.tiles.of(hue);
            let ratio = contrast(tile.ink, tile.fill);
            assert!(ratio >= 3.0, "{theme:?} {hue:?}: {ratio:.2}:1");
        }
    }
}

#[test]
fn a_glyph_reads_on_its_tile_with_a_pale_desktop_accent() {
    let mut pale = Palette::fallback(Theme::Light);
    pale.accent = Color32::from_rgb(0xb8, 0xd8, 0xf8);
    let tile = Tokens::derive(&pale, Theme::Light).tiles.of(Hue::Accent);

    assert!(contrast(tile.ink, tile.fill) >= 3.0);
}

#[test]
fn a_tile_stands_out_from_the_card_of_its_row() {
    for (theme, tokens) in both() {
        for hue in HUES {
            let ratio = contrast(tokens.tiles.of(hue).fill, tokens.card);
            assert!(ratio >= 1.15, "{theme:?} {hue:?}: {ratio:.2}:1");
        }
    }
}

#[test]
fn each_part_of_the_app_has_a_tile_of_its_own() {
    for (theme, tokens) in both() {
        let fills = HUES.map(|hue| tokens.tiles.of(hue).fill);
        for (index, fill) in fills.iter().enumerate() {
            assert!(
                !fills[index + 1..].contains(fill),
                "{theme:?}: two hues share the fill {fill:?}"
            );
        }
    }
}

#[test]
fn the_accent_tile_follows_the_desktop_accent() {
    let mut desktop = Palette::fallback(Theme::Dark);
    let before = Tokens::derive(&desktop, Theme::Dark).tiles.of(Hue::Accent);
    desktop.accent = Color32::from_rgb(0x3d, 0xae, 0xe9);
    let after = Tokens::derive(&desktop, Theme::Dark).tiles.of(Hue::Accent);

    assert_ne!(before.fill, after.fill);
}

#[test]
fn the_page_hue_is_the_accent_until_the_shell_sets_one() {
    let ctx = egui::Context::default();
    assert_eq!(page_hue(&ctx), Hue::Accent);

    set_page_hue(&ctx, Hue::Green);
    assert_eq!(page_hue(&ctx), Hue::Green);
}

fn statuses(tokens: &Tokens) -> [(&'static str, Color32, Color32); 4] {
    [
        ("error", tokens.error, tokens.banner_wash(tokens.error)),
        (
            "warning",
            tokens.warning,
            tokens.banner_wash(tokens.warning),
        ),
        (
            "success",
            tokens.success,
            tokens.banner_wash(tokens.success),
        ),
        ("accent", tokens.accent, tokens.accent_soft),
    ]
}

#[test]
fn a_status_glyph_keeps_its_colour_at_three_to_one_on_its_soft_tile() {
    for (theme, tokens) in both() {
        for (name, colour, _) in statuses(&tokens) {
            let tile = tokens.status_tile(colour);
            let ratio = contrast(tile.ink, tile.fill);

            assert_eq!(tile.ink, colour, "{theme:?} {name}");
            assert!(ratio >= 3.0, "{theme:?} {name}: {ratio:.2}:1");
        }
    }
}

#[test]
fn a_status_tile_is_a_wash_that_stands_out_on_its_banner() {
    for (theme, tokens) in both() {
        for (name, colour, banner) in statuses(&tokens) {
            let tile = tokens.status_tile(colour);
            let on_card = contrast(tile.fill, tokens.card);
            let on_banner = contrast(tile.fill, banner);

            assert!(on_card < contrast(colour, tokens.card), "{theme:?} {name}");
            assert!(
                on_card >= 1.15,
                "{theme:?} {name}: {on_card:.2}:1 on the card"
            );
            assert!(
                on_banner >= 1.08,
                "{theme:?} {name}: {on_banner:.2}:1 on the banner"
            );
        }
    }
}

#[test]
fn a_palette_with_tile_hues_of_its_own_tints_the_tiles_with_them() {
    let blue = Color32::from_rgb(0x81, 0xa1, 0xc1);
    let own = TileHues {
        blue,
        ..TileHues::of(Theme::Dark)
    };

    let palette = Palette {
        tiles: Some(own),
        ..Palette::fallback(Theme::Dark)
    };

    let ours = Tokens::derive(&palette, Theme::Dark).tiles.of(Hue::Blue);
    let fallback = Tokens::derive(&Palette::fallback(Theme::Dark), Theme::Dark)
        .tiles
        .of(Hue::Blue);

    assert_ne!(ours.fill, fallback.fill);
    assert_eq!(
        Tokens::derive(&palette, Theme::Dark).tiles.of(Hue::Green),
        fallback_green()
    );
}

fn fallback_green() -> super::super::tiles::Tile {
    Tokens::derive(&Palette::fallback(Theme::Dark), Theme::Dark)
        .tiles
        .of(Hue::Green)
}
