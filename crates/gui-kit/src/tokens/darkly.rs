//! The colour formulas of Darkly 0.5.45, the Qt style the app follows, with
//! the line of each in `kstyle/`. Darkly's literal black and white are the
//! theme's shadow shade and text, see [`super::shade`].

use egui::Color32;

use super::mix;

/// `qGray`, Qt's luma of a colour, as `(11 R + 16 G + 5 B) / 32`.
fn gray(colour: Color32) -> u32 {
    (11 * u32::from(colour.r()) + 16 * u32::from(colour.g()) + 5 * u32::from(colour.b())) / 32
}

/// `Helper::isDarkTheme` (darklyhelper.h:393): the window's grey at 110 or
/// below. It is the window that decides, not the app's theme.
pub(super) fn is_dark_theme(window: Color32) -> bool {
    gray(window) <= 110
}

/// `Helper::separatorColor` (darklyhelper.cpp:403): white at 16 on a dark
/// window, black at 16 on a light one.
pub(super) fn separator(window: Color32, text: Color32, shadow: Color32) -> Color32 {
    alpha(if is_dark_theme(window) { text } else { shadow }, 16)
}

/// `Helper::sidePanelOutlineColor` (darklyhelper.cpp:173): black at 20 on a
/// window above grey 150, else at 50. Darkly's 150 is not the 110 of
/// [`is_dark_theme`], as Darkly keeps both.
pub(super) fn sidebar_edge(window: Color32, shadow: Color32) -> Color32 {
    alpha(shadow, if gray(window) > 150 { 20 } else { 50 })
}

/// `Helper::frameBackgroundColor` (darklyhelper.cpp:180), the fill of a
/// group box, a card. Darkly draws it without a border.
pub(super) fn frame_fill(window: Color32, view: Color32) -> Color32 {
    mix(window, view, 0.3)
}

/// `Helper::topHighlight` (darklyhelper.cpp:857): the default colour of
/// darklyhelper.h:247, white at 30 of 255, drawn on a dark window only.
/// Literal white is the light end.
pub(super) fn top_highlight(window: Color32, light: Color32) -> Color32 {
    if is_dark_theme(window) {
        alpha(light, 30)
    } else {
        Color32::TRANSPARENT
    }
}

/// Darkly's `alphaColor` (darklyhelper.cpp:423) for a share of 255. egui
/// stores the fill premultiplied, as every tinted fill is.
pub(super) fn alpha(colour: Color32, of_255: u8) -> Color32 {
    colour.gamma_multiply(f32::from(of_255) / 255.0)
}

/// Qt's `QColor::darker`: the HSV value divided by `factor`, so 1.1 is
/// Qt's `darker(110)`.
pub fn darker(colour: Color32, factor: f32) -> Color32 {
    let (sixths, saturation, value) = hsv(colour);
    with_alpha(colour, from_hsv(sixths, saturation, value / factor))
}

/// Qt's `QColor::lighter`: the HSV value multiplied by `factor`, so 1.1 is
/// Qt's `lighter(110)`.
///
/// A value past full brightness is held at full, and the overflow takes
/// saturation off instead, which keeps greys grey.
pub fn lighter(colour: Color32, factor: f32) -> Color32 {
    let (sixths, mut saturation, value) = hsv(colour);
    let lit = value * factor;
    let bright = if lit > 1.0 {
        saturation = (saturation - (lit - 1.0)).max(0.0);
        1.0
    } else {
        lit
    };

    with_alpha(colour, from_hsv(sixths, saturation, bright))
}

/// Qt's HSV of a colour, in the sRGB values as they are. egui's `Hsva`
/// works in linear light, which would move every step Qt takes.
#[expect(
    clippy::float_cmp,
    reason = "Qt picks the sector by the exact equality of the channels, as this does"
)]
fn hsv(colour: Color32) -> (f32, f32, f32) {
    let [red, green, blue] =
        [colour.r(), colour.g(), colour.b()].map(|value| f32::from(value) / 255.0);

    let max = red.max(green).max(blue);
    let delta = max - red.min(green).min(blue);
    let saturation = if max > 0.0 { delta / max } else { 0.0 };
    let sixths = if delta == 0.0 {
        0.0
    } else if max == red {
        ((green - blue) / delta).rem_euclid(6.0)
    } else if max == green {
        (blue - red) / delta + 2.0
    } else {
        (red - green) / delta + 4.0
    };

    (sixths, saturation, max)
}

/// The sRGB colour of a hue in sixths of a turn, a saturation and a value,
/// each channel `value * (1 - saturation * shade)` with the usual shade.
fn from_hsv(sixths: f32, saturation: f32, value: f32) -> Color32 {
    let channel = |offset: f32| {
        let position = (offset + sixths).rem_euclid(6.0);
        let shade = position.min(4.0 - position).clamp(0.0, 1.0);
        channel_byte(value * saturation.mul_add(-shade, 1.0))
    };

    opaque_rgb(channel(5.0), channel(3.0), channel(1.0))
}

#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the value is clamped to 0 to 1 first, so it fits a byte"
)]
fn channel_byte(share: f32) -> u8 {
    (share.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// A colour the code derives from the palette's own, never a new hue.
#[expect(
    clippy::disallowed_methods,
    reason = "the colour is derived from the palette's colours, not new"
)]
pub(super) const fn opaque_rgb(red: u8, green: u8, blue: u8) -> Color32 {
    Color32::from_rgb(red, green, blue)
}

fn with_alpha(source: Color32, colour: Color32) -> Color32 {
    colour.gamma_multiply(f32::from(source.a()) / 255.0)
}
