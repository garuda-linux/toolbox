#![expect(
    clippy::disallowed_methods,
    reason = "the tests build the accent colours they check"
)]

use std::sync::Arc;

use egui::Color32;
use toolbox_gui_kit::oklch::hue_chroma;

use super::{bytes, svg};

const RED: Color32 = Color32::from_rgb(0xc0, 0x30, 0x30);
const SOURCE_FACETS: [&str; 3] = ["#7f40c0", "#7f3fbf", "#7f72f2"];

fn first_stop(svg: &str) -> Color32 {
    let at = svg.find("stop-color:#").expect("a stop") + "stop-color:".len();
    let hex = svg.as_bytes().get(at..at + 7).expect("seven bytes");
    let channel = |offset: usize| {
        u8::from_str_radix(std::str::from_utf8(&hex[offset..offset + 2]).unwrap(), 16).unwrap()
    };

    Color32::from_rgb(channel(1), channel(3), channel(5))
}

#[test]
fn a_red_accent_leaves_no_violet_behind() {
    let logo = svg(RED, false);
    for facet in SOURCE_FACETS {
        assert!(!logo.contains(facet), "{facet} is still in the logo");
    }
}

#[test]
fn the_black_and_white_details_stay() {
    let logo = svg(RED, false);
    assert!(logo.contains("#000000") && logo.contains("#ffffff"));
}

#[test]
fn the_dark_copy_is_lighter_than_the_light_one() {
    let light =
        |colour: Color32| u32::from(colour.r()) + u32::from(colour.g()) + u32::from(colour.b());

    assert!(light(first_stop(&svg(RED, true))) > light(first_stop(&svg(RED, false))));
}

#[test]
fn the_facets_take_the_hue_of_the_accent() {
    let (hue, _) = hue_chroma(RED);
    let (stop_hue, _) = hue_chroma(first_stop(&svg(RED, false)));
    let off = (stop_hue - hue).rem_euclid(360.0);
    assert!(
        off.min(360.0 - off) < 10.0,
        "stop hue {stop_hue} is far from the accent's {hue}"
    );
}

#[test]
fn the_same_colours_come_from_the_cache() {
    let ctx = egui::Context::default();
    let first = bytes(&ctx, RED, false);
    assert!(Arc::ptr_eq(&first, &bytes(&ctx, RED, false)));
}

#[test]
fn a_new_accent_replaces_the_cached_logo() {
    let ctx = egui::Context::default();
    let red = bytes(&ctx, RED, false);
    let blue = bytes(&ctx, Color32::from_rgb(0x30, 0x60, 0xc0), false);
    assert!(!Arc::ptr_eq(&red, &blue));
}
