//! KDE's shadow shade, the Qt palette's Shadow role, as KDE computes it.
//!
//! `KColorScheme::shade` (kcolorscheme.cpp:566 to 612) with the role
//! `ShadowShade`, on the hue, chroma and luma space of kguiaddons
//! (`KHCY`, kcolorspaces.cpp:49 to 160). Its contrast is `[KDE] contrast`
//! times 0.1, 0.7 when absent (kcolorscheme.cpp:519 to 526).

#![expect(
    clippy::suboptimal_flops,
    clippy::float_cmp,
    reason = "KDE's formulas are kept term by term, so each step rounds as the library's does"
)]

use egui::Color32;

use super::darkly::opaque_rgb;

/// Rec. 709 luma weights, the ones kcolorspaces.cpp:29 selects.
const YC: [f32; 3] = [0.2126, 0.7152, 0.0722];

/// KDE's default contrast when `[KDE] contrast` is absent.
const DEFAULT_CONTRAST: i8 = 7;

#[derive(Clone, Copy)]
struct Khcy {
    hue: f32,
    chroma: f32,
    luma: f32,
}

impl Khcy {
    /// `KHCY::KHCY(const QColor &)` (kcolorspaces.cpp:57 to 87).
    fn of(colour: Color32) -> Self {
        let [red, green, blue] =
            [colour.r(), colour.g(), colour.b()].map(|value| gamma(f32::from(value) / 255.0));

        let luma = YC[0] * red + YC[1] * green + YC[2] * blue;

        let high = red.max(green).max(blue);
        let low = red.min(green).min(blue);
        let span = 6.0 * (high - low);
        let hue = if low == high {
            0.0
        } else if red == high {
            (green - blue) / span
        } else if green == high {
            (blue - red) / span + 1.0 / 3.0
        } else {
            (red - green) / span + 2.0 / 3.0
        };

        let chroma = if red == green && green == blue {
            0.0
        } else {
            ((luma - low) / luma).max((high - luma) / (1.0 - luma))
        };

        Self { hue, chroma, luma }
    }

    /// `KHCY::qColor` (kcolorspaces.cpp:89 to 157), the channels in the
    /// order the hue's sector puts them.
    fn colour(self) -> Color32 {
        let hue = wrap(self.hue);
        let chroma = normalize(self.chroma);
        let luma = normalize(self.luma);

        let sixths = hue * 6.0;
        let (sector, mid) = if sixths < 1.0 {
            (sixths, YC[0] + YC[1] * sixths)
        } else if sixths < 2.0 {
            (2.0 - sixths, YC[1] + YC[0] * (2.0 - sixths))
        } else if sixths < 3.0 {
            (sixths - 2.0, YC[1] + YC[2] * (sixths - 2.0))
        } else if sixths < 4.0 {
            (4.0 - sixths, YC[2] + YC[1] * (4.0 - sixths))
        } else if sixths < 5.0 {
            (sixths - 4.0, YC[2] + YC[0] * (sixths - 4.0))
        } else {
            (6.0 - sixths, YC[0] + YC[2] * (6.0 - sixths))
        };

        let (high, middle, low) = if mid >= luma {
            (
                luma + luma * chroma * (1.0 - mid) / mid,
                luma + luma * chroma * (sector - mid) / mid,
                luma - luma * chroma,
            )
        } else {
            (
                luma + (1.0 - luma) * chroma,
                luma + (1.0 - luma) * chroma * (sector - mid) / (1.0 - mid),
                luma - (1.0 - luma) * chroma * mid / (1.0 - mid),
            )
        };

        let [red, green, blue] = if sixths < 1.0 {
            [high, middle, low]
        } else if sixths < 2.0 {
            [middle, high, low]
        } else if sixths < 3.0 {
            [low, high, middle]
        } else if sixths < 4.0 {
            [low, middle, high]
        } else if sixths < 5.0 {
            [middle, low, high]
        } else {
            [high, low, middle]
        };

        channel_colour([red, green, blue].map(igamma))
    }
}

/// `KColorScheme::shade(window, ShadowShade, contrast)`, with the contrast
/// read from `[KDE] contrast` on its 0 to 10 scale.
pub(super) fn shadow_shade(window: Color32, contrast: Option<i8>) -> Color32 {
    let contrast = f32::from(contrast.unwrap_or(DEFAULT_CONTRAST)) * 0.1;
    let contrast = contrast.clamp(-1.0, 1.0);
    let luma = Khcy::of(window).luma;

    // Very dark (kcolorscheme.cpp:576) and very light colours (:591)
    // take their own steps.
    if luma < 0.006 {
        return shade(window, 0.03 + 0.60 * contrast, 0.0);
    }

    if luma > 0.93 {
        return shade(window, -0.10 - 0.90 * contrast, 0.0);
    }

    let dark_amount = (-luma) * (0.55 + contrast * 0.35);
    darken(shade(window, dark_amount, 0.0), 0.5 + 0.3 * luma, 1.0)
}

/// `KColorUtils::shade` (kcolorutils.cpp:89): luma and chroma moved by
/// steps, each held within 0 to 1.
fn shade(colour: Color32, luma: f32, chroma: f32) -> Color32 {
    let mut khcy = Khcy::of(colour);
    khcy.luma = normalize(khcy.luma + luma);
    khcy.chroma = normalize(khcy.chroma + chroma);
    khcy.colour()
}

/// `KColorUtils::darken` (kcolorutils.cpp:81): luma and chroma scaled.
fn darken(colour: Color32, luma: f32, chroma: f32) -> Color32 {
    let mut khcy = Khcy::of(colour);
    khcy.luma = normalize(khcy.luma * (1.0 - luma));
    khcy.chroma = normalize(khcy.chroma * chroma);
    khcy.colour()
}

/// `normalize` (`kguiaddons_colorhelpers_p.h`:13) holds a value within 0 to 1.
fn normalize(value: f32) -> f32 {
    if value < 1.0 {
        if value > 0.0 { value } else { 0.0 }
    } else {
        1.0
    }
}

/// `KHCY::gamma` (kcolorspaces.cpp:34) and its inverse.
fn gamma(value: f32) -> f32 {
    normalize(value).powf(2.2)
}

fn igamma(value: f32) -> f32 {
    normalize(value).powf(1.0 / 2.2)
}

/// `wrap` (kcolorspaces.cpp:16): the hue within one turn.
fn wrap(value: f32) -> f32 {
    let rest = value % 1.0;
    if rest < 0.0 {
        1.0 + rest
    } else if rest > 0.0 {
        rest
    } else {
        0.0
    }
}

#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "each channel is held within 0 to 1 first, so it fits a byte"
)]
fn channel_colour([red, green, blue]: [f32; 3]) -> Color32 {
    let byte = |share: f32| (normalize(share) * 255.0).round() as u8;
    opaque_rgb(byte(red), byte(green), byte(blue))
}
