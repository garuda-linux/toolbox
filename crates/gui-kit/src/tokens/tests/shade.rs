use egui::Color32;

use super::super::shade::shadow_shade;
use crate::desktop::kde::parse;
use crate::testing::desktop::kde::{BREEZEDARK_COLORS, INSTALLED_KDEGLOBALS};

fn within_one(colour: Color32, expected: [u8; 3]) -> bool {
    [colour.r(), colour.g(), colour.b()]
        .iter()
        .zip(expected)
        .all(|(&got, want)| got.abs_diff(want) <= 1)
}

/// `KColorScheme::shade(color, KColorScheme::ShadowShade, contrast)` of KDE
/// Frameworks 6.32 (libKF6ColorScheme, run from a C++ harness on these
/// inputs), which the Qt palette's Shadow role takes.
#[test]
fn shadow_shade_is_kcolorschemes_shadow_shade() {
    let cases = [
        // The Mokka window at the contrast its capture names (4).
        (Color32::from_rgb(30, 30, 46), Some(4), [13, 13, 20]),
        // The Breeze Dark window at its capture's contrast (4).
        (Color32::from_rgb(32, 35, 38), Some(4), [14, 15, 16]),
        // A light window at KDE's default contrast of 7, the branch for
        // a mid-light colour.
        (Color32::from_rgb(239, 241, 245), None, [52, 61, 76]),
        // The very dark and the very light branches.
        (Color32::from_rgb(0, 0, 0), None, [177, 177, 177]),
        (Color32::from_rgb(255, 255, 255), None, [141, 141, 141]),
    ];

    for (window, contrast, expected) in cases {
        let shade = shadow_shade(window, contrast);
        assert!(
            within_one(shade, expected),
            "{window:?} at {contrast:?}: {shade:?}, KDE gives {expected:?}"
        );
    }
}

#[test]
fn shadow_shade_of_the_captured_desktops_reads_their_own_contrast() {
    let mokka = parse(INSTALLED_KDEGLOBALS).colors;
    let breeze = parse(BREEZEDARK_COLORS).colors;

    let mokka_shade = shadow_shade(mokka.window.expect("named"), mokka.contrast);
    assert!(within_one(mokka_shade, [13, 13, 20]), "{mokka_shade:?}");

    let breeze_shade = shadow_shade(breeze.window.expect("named"), breeze.contrast);
    assert!(within_one(breeze_shade, [14, 15, 16]), "{breeze_shade:?}");
}

#[test]
fn an_edited_contrast_changes_the_shade() {
    // The Mokka capture with its contrast set to 10 instead of 4. KDE gives
    // (8, 8, 12) for that window at contrast 1.0.
    let edited = INSTALLED_KDEGLOBALS.replace("contrast=4", "contrast=10");
    let colors = parse(&edited).colors;

    let shade = shadow_shade(colors.window.expect("named"), colors.contrast);
    assert!(within_one(shade, [8, 8, 12]), "{shade:?}");
    assert_ne!(shade, shadow_shade(colors.window.expect("named"), Some(4)));
}
