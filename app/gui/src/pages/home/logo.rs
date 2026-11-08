//! The logo in the desktop's accent. Its violet facets turn to the accent's
//! hue and keep their lightness, so the logo keeps its look on any desktop
//! colour. The dark copy is lightened for a dark surface.

#[cfg(test)]
mod tests;

use std::sync::Arc;

use egui::{Color32, Context, Id, Image};
use toolbox_gui_kit::oklch::{hue_chroma, shift_hue};

const SOURCE: &str = include_str!("../../../assets/garuda-logo.svg");
/// The violet the logo is drawn in, before any accent turns it.
const SOURCE_VIOLET: (u8, u8, u8) = (0x7f, 0x40, 0xc0);
/// A grey accent has no hue to turn to, so the logo keeps its own.
const MIN_CHROMA: f32 = 0.04;
/// How far the dark copy moves toward white.
const LIGHTEN: f32 = 0.25;
/// A colour written as `#rrggbb`.
const TOKEN_LEN: usize = 7;

#[derive(Clone, Debug, Default)]
struct Slot {
    uri: String,
    bytes: Arc<[u8]>,
}

/// The logo for `accent`, the dark copy on a dark surface.
pub fn image(ctx: &Context, accent: Color32, dark: bool) -> Image<'static> {
    Image::from_bytes(uri(accent, dark), bytes(ctx, accent, dark))
}

/// The SVG of [`image`], kept in the context until the accent or the theme
/// changes, so the colours are turned once and not on every frame.
fn bytes(ctx: &Context, accent: Color32, dark: bool) -> Arc<[u8]> {
    let uri = uri(accent, dark);
    ctx.data_mut(|data| {
        let slot = data.get_temp_mut_or_default::<Slot>(slot_id());
        if slot.uri != uri {
            slot.uri = uri;
            slot.bytes = Arc::from(svg(accent, dark).into_bytes());
        }

        Arc::clone(&slot.bytes)
    })
}

fn slot_id() -> Id {
    Id::unique("toolbox logo")
}

fn uri(accent: Color32, dark: bool) -> String {
    format!(
        "bytes://toolbox/logo-{}-{:02x}{:02x}{:02x}.svg",
        if dark { "dark" } else { "light" },
        accent.r(),
        accent.g(),
        accent.b()
    )
}

/// The logo's SVG with its colours turned to `accent`. Black and white
/// details stay as they are.
fn svg(accent: Color32, dark: bool) -> String {
    let (hue, chroma) = hue_chroma(accent);
    let turn = if chroma < MIN_CHROMA {
        0.0
    } else {
        hue - source_hue()
    };

    recolour(SOURCE, |colour| {
        let turned = shift_hue(colour, turn);
        if dark {
            turned.lerp_to_gamma(Color32::WHITE, LIGHTEN)
        } else {
            turned
        }
    })
}

fn source_hue() -> f32 {
    hue_chroma(source_violet()).0
}

#[expect(
    clippy::disallowed_methods,
    reason = "the logo's own colour, read from its file, not a palette colour"
)]
const fn source_violet() -> Color32 {
    let (red, green, blue) = SOURCE_VIOLET;
    Color32::from_rgb(red, green, blue)
}

/// Every `#rrggbb` in `source` but black and white goes through `turn`.
/// Everything else is copied byte for byte, so the text stays valid.
fn recolour(source: &str, turn: impl Fn(Color32) -> Color32) -> String {
    let bytes = source.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        let token = bytes
            .get(index..index + TOKEN_LEN)
            .filter(|token| is_colour(token));

        match token {
            Some(token) if token != b"#000000" && token != b"#ffffff" => {
                let new = turn(parse(token));
                out.extend_from_slice(hex(new).as_bytes());
                index += TOKEN_LEN;
            }
            _ => {
                out.push(byte);
                index += 1;
            }
        }
    }

    String::from_utf8_lossy(&out).into_owned()
}

fn hex(colour: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", colour.r(), colour.g(), colour.b())
}

fn is_colour(token: &[u8]) -> bool {
    token.first() == Some(&b'#')
        && token
            .get(1..)
            .is_some_and(|digits| digits.iter().all(u8::is_ascii_hexdigit))
}

#[expect(
    clippy::disallowed_methods,
    reason = "the logo's own colour, read from its file, not a palette colour"
)]
fn parse(token: &[u8]) -> Color32 {
    let channel = |at: usize| {
        token
            .get(at..at + 2)
            .and_then(|digits| std::str::from_utf8(digits).ok())
            .and_then(|digits| u8::from_str_radix(digits, 16).ok())
            .unwrap_or_default()
    };

    Color32::from_rgb(channel(1), channel(3), channel(5))
}
