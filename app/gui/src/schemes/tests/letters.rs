use egui::Color32;
use toolbox_gui_kit::oklch::hue_chroma;
use toolbox_gui_kit::tokens::{LETTER_HUES, Tokens, contrast};

use super::{shipped, tokens_of};

/// The angle between two hues, 0 to 180.
fn gap(one: f32, other: f32) -> f32 {
    let turn = (one - other).rem_euclid(360.0);
    turn.min(360.0 - turn)
}

fn fills(tokens: &Tokens) -> Vec<Color32> {
    (0..LETTER_HUES)
        .map(|index| tokens.tiles.letters().of(index).fill)
        .collect()
}

fn mean_chroma(id: &str) -> f32 {
    let catalog = shipped();
    let scheme = catalog
        .schemes()
        .iter()
        .find(|scheme| scheme.id().as_str() == id)
        .unwrap();

    let chroma: f32 = fills(&tokens_of(scheme))
        .into_iter()
        .map(|fill| hue_chroma(fill).1)
        .sum();

    let count = f32::from(u8::try_from(LETTER_HUES).unwrap());
    chroma / count
}

#[test]
fn every_scheme_s_letters_read_on_tiles_set_apart() {
    for scheme in shipped().schemes() {
        let tokens = tokens_of(scheme);
        let name = scheme.id().as_str();
        let letters = tokens.tiles.letters();

        for index in 0..LETTER_HUES {
            let tile = letters.of(index);
            let ratio = contrast(tile.ink, tile.fill);
            assert!(ratio >= 4.5, "{name} hue {index}: letter {ratio:.2}:1");

            let apart = contrast(tile.fill, tokens.card);
            assert!(apart >= 1.5, "{name} hue {index}: {apart:.2}:1 on the card");
        }

        let hues: Vec<f32> = fills(&tokens)
            .into_iter()
            .map(|fill| hue_chroma(fill).0)
            .collect();

        for (index, hue) in hues.iter().enumerate() {
            for other in hues.iter().skip(index + 1) {
                assert!(
                    gap(*hue, *other) >= 12.0,
                    "{name}: hues {hue:.0} and {other:.0} are too close"
                );
            }
        }
    }
}

#[test]
fn nord_s_tiles_are_softer_than_dracula_s_and_sweet_s() {
    let nord = mean_chroma("nord");
    for vivid in ["dracula", "sweet"] {
        let chroma = mean_chroma(vivid);
        assert!(
            nord + 0.02 < chroma,
            "Nord at chroma {nord:.3}, {vivid} at {chroma:.3}"
        );
    }
}
