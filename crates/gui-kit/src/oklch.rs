//! Hue and lightness changes in OKLCH, the space designers scale colours
//! in. What is not changed stays put, so a moved colour keeps its weight
//! on the page.

#[cfg(test)]
mod tests;

use egui::Color32;
use egui::ecolor::{gamma_u8_from_linear_f32, linear_f32_from_gamma_u8};

/// Steps of the search for the most chroma that still fits sRGB.
const FIT_STEPS: u32 = 16;

/// The hue angle of `colour` in degrees, 0 to 360, and its chroma.
pub fn hue_chroma(colour: Color32) -> (f32, f32) {
    let [_, green_red, blue_yellow] = to_oklab(colour);
    let hue = blue_yellow.atan2(green_red).to_degrees().rem_euclid(360.0);
    (hue, green_red.hypot(blue_yellow))
}

/// `colour` turned `degrees` round the hue wheel, with its lightness and
/// alpha kept. Where the turned colour leaves sRGB, its chroma is reduced
/// until it fits, so the hue stays where it was turned to.
pub fn shift_hue(colour: Color32, degrees: f32) -> Color32 {
    let [lightness, green_red, blue_yellow] = to_oklab(colour);
    let chroma = green_red.hypot(blue_yellow);
    let hue = blue_yellow.atan2(green_red) + degrees.to_radians();
    let (sin, cos) = hue.sin_cos();

    let fitted = fit([lightness, chroma * cos, chroma * sin]);
    let turned = to_srgb(oklab_to_linear(fitted));
    turned.gamma_multiply(f32::from(colour.a()) / 255.0)
}

/// The OKLCH lightness of `colour`, 0 black to 1 white.
pub fn lightness(colour: Color32) -> f32 {
    to_oklab(colour)[0]
}

/// `colour` at `lightness`, its hue and alpha kept. Its chroma too, as
/// far as sRGB holds it at that lightness.
pub fn with_lightness(colour: Color32, lightness: f32) -> Color32 {
    let [_, green_red, blue_yellow] = to_oklab(colour);
    let fitted = fit([lightness.clamp(0.0, 1.0), green_red, blue_yellow]);
    to_srgb(oklab_to_linear(fitted)).gamma_multiply(f32::from(colour.a()) / 255.0)
}

/// The opaque colour at OKLCH `lightness`, `chroma` and `hue` in degrees.
/// Where it leaves sRGB, its chroma is reduced until it fits, so its hue
/// and lightness stay.
pub fn from_oklch(lightness: f32, chroma: f32, hue: f32) -> Color32 {
    let (sin, cos) = hue.to_radians().sin_cos();
    let fitted = fit([lightness.clamp(0.0, 1.0), chroma * cos, chroma * sin]);
    to_srgb(oklab_to_linear(fitted))
}

fn to_oklab(colour: Color32) -> [f32; 3] {
    let [red, green, blue] = [colour.r(), colour.g(), colour.b()].map(linear_f32_from_gamma_u8);
    let long = 0.412_221_46_f32.mul_add(red, 0.536_332_54_f32.mul_add(green, 0.051_445_99 * blue));
    let medium = 0.211_903_5_f32.mul_add(red, 0.680_699_5_f32.mul_add(green, 0.107_396_96 * blue));
    let short = 0.088_302_46_f32.mul_add(red, 0.281_718_84_f32.mul_add(green, 0.629_978_7 * blue));
    let [long_root, medium_root, short_root] = [long.cbrt(), medium.cbrt(), short.cbrt()];

    [
        0.210_454_26_f32.mul_add(
            long_root,
            0.793_617_8_f32.mul_add(medium_root, -0.004_072_047 * short_root),
        ),
        1.977_998_5_f32.mul_add(
            long_root,
            (-2.428_592_2_f32).mul_add(medium_root, 0.450_593_7 * short_root),
        ),
        0.025_904_037_f32.mul_add(
            long_root,
            0.782_771_76_f32.mul_add(medium_root, -0.808_675_77 * short_root),
        ),
    ]
}

/// Linear `sRGB` of an `OKLab` colour, whether or not it fits `sRGB`.
fn oklab_to_linear([lightness, green_red, blue_yellow]: [f32; 3]) -> [f32; 3] {
    let long = 0.396_337_78_f32
        .mul_add(green_red, 0.215_803_76_f32.mul_add(blue_yellow, lightness))
        .powi(3);

    let medium = (-0.105_561_346_f32)
        .mul_add(
            green_red,
            (-0.063_854_17_f32).mul_add(blue_yellow, lightness),
        )
        .powi(3);

    let short = (-0.089_484_18_f32)
        .mul_add(
            green_red,
            (-1.291_485_5_f32).mul_add(blue_yellow, lightness),
        )
        .powi(3);

    [
        4.076_741_7_f32.mul_add(
            long,
            (-3.307_711_6_f32).mul_add(medium, 0.230_969_93 * short),
        ),
        (-1.268_438_f32).mul_add(long, 2.609_757_4_f32.mul_add(medium, -0.341_319_4 * short)),
        (-0.004_196_086_3_f32).mul_add(
            long,
            (-0.703_418_6_f32).mul_add(medium, 1.707_614_7 * short),
        ),
    ]
}

/// The `OKLab` colour with the most chroma, at its lightness and hue, that
/// lies inside `sRGB`.
fn fit(lab: [f32; 3]) -> [f32; 3] {
    if fits(oklab_to_linear(lab)) {
        return lab;
    }

    let [lightness, green_red, blue_yellow] = lab;
    let at = |scale: f32| [lightness, green_red * scale, blue_yellow * scale];
    let (mut inside, mut outside) = (0.0_f32, 1.0_f32);
    for _ in 0..FIT_STEPS {
        let middle = inside.midpoint(outside);
        if fits(oklab_to_linear(at(middle))) {
            inside = middle;
        } else {
            outside = middle;
        }
    }

    at(inside)
}

fn fits(linear: [f32; 3]) -> bool {
    linear
        .iter()
        .all(|channel| (-1e-4..=1.0 + 1e-4).contains(channel))
}

#[expect(
    clippy::disallowed_methods,
    reason = "the colour is computed from the colour it turns, not written in the code"
)]
fn to_srgb(linear: [f32; 3]) -> Color32 {
    let [red, green, blue] = linear.map(gamma_u8_from_linear_f32);
    Color32::from_rgb(red, green, blue)
}
