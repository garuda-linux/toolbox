//! The fonts: the desktop's when the app found their files, Inter and
//! JetBrains Mono behind them for every glyph the desktop's lack and on
//! desktops that name none.
//!
//! Each weight is a family of its own (egui picks no weights), and
//! [`font`] sizes text on the type scale relative to the body text.

mod cap;
#[cfg(test)]
mod tests;

use std::sync::Arc;

pub use cap::cap_centring;
use egui::epaint::text::{FontTweak, VariationCoords};
use egui::{Context, FontData, FontDefinitions, FontFamily, FontId, Style, TextStyle};
use skrifa::{FontRef, MetadataProvider as _, Tag};

pub const INTER: &str = "Inter";
pub const JETBRAINS_MONO: &str = "JetBrains Mono";

/// Page titles.
pub const TITLE: f32 = 2.3;
/// Section headings and dialog titles.
pub const SECTION: f32 = 1.4;
/// Labels of rows and buttons, the body size.
pub const BODY: f32 = 1.0;
/// Descriptions, counts and tags.
pub const SMALL: f32 = 0.92;

/// KDE's and GNOME's default size, used when the desktop names none.
const DEFAULT_POINTS: f32 = 10.0;
/// A point is 1/72 inch, egui's points are 1/96 inch at scale 1.
const PIXELS_PER_POINT: f32 = 96.0 / 72.0;

/// A desktop font file the app found and read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontFile {
    pub name: String,
    pub data: Vec<u8>,
    /// The face inside a font collection.
    pub index: u32,
    /// CSS weight, set on the weight axis of a variable font.
    pub weight: u16,
}

/// The weights the app draws text in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Weight {
    Regular,
    Medium,
    SemiBold,
    Bold,
}

impl Weight {
    pub const ALL: [Self; 4] = [Self::Regular, Self::Medium, Self::SemiBold, Self::Bold];

    pub const fn css(self) -> u16 {
        match self {
            Self::Regular => 400,
            Self::Medium => 500,
            Self::SemiBold => 600,
            Self::Bold => 700,
        }
    }

    pub const fn nearest(css: u16) -> Self {
        match css {
            0..450 => Self::Regular,
            450..550 => Self::Medium,
            550..650 => Self::SemiBold,
            _ => Self::Bold,
        }
    }

    /// Regular is egui's own proportional family, so plain text needs no
    /// family named.
    pub fn family(self) -> FontFamily {
        match self {
            Self::Regular => FontFamily::Proportional,
            Self::Medium => FontFamily::Name("medium".into()),
            Self::SemiBold => FontFamily::Name("semibold".into()),
            Self::Bold => FontFamily::Name("bold".into()),
        }
    }

    fn data_name(self, font: &str) -> String {
        match self {
            Self::Regular => font.to_owned(),
            Self::Medium | Self::SemiBold | Self::Bold => format!("{font} {}", self.css()),
        }
    }
}

/// Every font egui may draw with. `ui` holds the desktop's font at the
/// weights it has, each going first in the family of its weight.
#[must_use = "the definitions do nothing until set on the context"]
pub fn definitions(ui: &[FontFile], fixed: Option<&FontFile>) -> FontDefinitions {
    let mut fonts = FontDefinitions::empty();
    let inter = include_bytes!("../assets/fonts/InterVariable.ttf");

    for weight in Weight::ALL {
        let name = weight.data_name(INTER);
        let data = weighted(FontData::from_static(inter), weight.css());
        fonts.font_data.insert(name.clone(), Arc::new(data));

        let mut family = vec![name];
        if let Some(file) = ui
            .iter()
            .find(|file| Weight::nearest(file.weight) == weight)
        {
            let key = weight.data_name(&file.name);
            if !fonts.font_data.contains_key(&key) {
                fonts
                    .font_data
                    .insert(key.clone(), Arc::new(desktop_data(file)));
                family.insert(0, key);
            }
        }

        fonts.families.insert(weight.family(), family);
    }

    fonts.font_data.insert(
        JETBRAINS_MONO.to_owned(),
        Arc::new(weighted(
            FontData::from_static(include_bytes!("../assets/fonts/JetBrainsMono-Variable.ttf")),
            Weight::Regular.css(),
        )),
    );

    let mut monospace = vec![JETBRAINS_MONO.to_owned(), INTER.to_owned()];
    if let Some(file) = fixed.filter(|file| !fonts.font_data.contains_key(&file.name)) {
        fonts
            .font_data
            .insert(file.name.clone(), Arc::new(desktop_data(file)));
        monospace.insert(0, file.name.clone());
    }

    fonts.families.insert(FontFamily::Monospace, monospace);
    fonts
}

/// Puts `file` behind every family, for the glyphs no other font has,
/// such as emoji. A colour font draws in colour.
pub fn add_fallback(fonts: &mut FontDefinitions, file: &FontFile) {
    if fonts.font_data.contains_key(&file.name) {
        return;
    }

    fonts
        .font_data
        .insert(file.name.clone(), Arc::new(desktop_data(file)));
    for family in fonts.families.values_mut() {
        family.push(file.name.clone());
    }
}

/// Sizes the text styles after the desktop's point sizes, so text is as
/// large as in the desktop's other apps.
pub fn apply_sizes(ctx: &Context, ui_points: Option<f32>, fixed_points: Option<f32>) {
    let body = ui_points.unwrap_or(DEFAULT_POINTS) * PIXELS_PER_POINT;
    let mono = fixed_points.map_or(body, |points| points * PIXELS_PER_POINT);
    let sizes = [
        (TextStyle::Small, body * SMALL, Weight::Regular.family()),
        (TextStyle::Body, body, Weight::Regular.family()),
        (TextStyle::Button, body, Weight::Medium.family()),
        (TextStyle::Heading, body * SECTION, Weight::Bold.family()),
        (TextStyle::Monospace, mono, FontFamily::Monospace),
    ];

    ctx.all_styles_mut(|style| {
        for (text_style, size, family) in sizes.clone() {
            style
                .text_styles
                .insert(text_style, FontId::new(size, family));
        }
    });
}

pub fn font(style: &Style, weight: Weight, scale: f32) -> FontId {
    let body = style
        .text_styles
        .get(&TextStyle::Body)
        .map_or(DEFAULT_POINTS * PIXELS_PER_POINT, |font| font.size);

    FontId::new(body * scale, weight.family())
}

/// Whether the font draws any weight through its weight axis, so one file
/// serves every weight.
pub fn has_weight_axis(data: &[u8], index: u32) -> bool {
    FontRef::from_index(data, index).is_ok_and(|font| {
        font.axes()
            .iter()
            .any(|axis| axis.tag() == Tag::new(b"wght"))
    })
}

fn desktop_data(file: &FontFile) -> FontData {
    let mut data = FontData::from_owned(file.data.clone());
    data.index = file.index;
    weighted(data, file.weight)
}

/// Sets the weight axis, which a font without one ignores, and moves the
/// glyphs so their capitals sit in the middle of the line, where egui
/// centres the line box (see [`cap_centring`]).
fn weighted(data: FontData, weight: u16) -> FontData {
    let shift = cap_centring((*data.font).as_ref(), data.index);
    data.tweak(FontTweak {
        y_offset_factor: shift,
        coords: VariationCoords::new([("wght", f32::from(weight))]),
        ..FontTweak::default()
    })
}
