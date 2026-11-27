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

/// A name or a figure set large, such as About's name, larger than an even step.
pub const TITLE: f32 = 2.3;
/// Kirigami's level 1 heading (`Heading.qml`), the page title in the toolbar.
pub const PAGE_TITLE: f32 = 1.35;
/// Kirigami's level 2 heading (`Heading.qml`), the section headings.
pub const SECTION_TITLE: f32 = 1.2;
/// egui's Heading style.
pub const SECTION: f32 = 1.4;
/// Labels of rows and buttons, pane titles, the body size.
pub const BODY: f32 = 1.0;
/// Descriptions, counts and the Console's state. No text takes a size off
/// these steps.
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

    /// KDE and GNOME draw labels, buttons and body text in the desktop's
    /// own weight, so Regular, Medium and `SemiBold` share its family and
    /// only Bold steps up from it (Kirigami's headings and emphasis).
    pub fn family(self) -> FontFamily {
        match self {
            Self::Regular | Self::Medium | Self::SemiBold => FontFamily::Proportional,
            Self::Bold => FontFamily::Name("bold".into()),
        }
    }
}

/// The desktop's UI font at the two steps the app draws: its own weight for
/// the regular text, and its bold face for the bold step.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiFaces {
    pub regular: Option<FontFile>,
    pub bold: Option<FontFile>,
}

/// The name egui knows a face by: the family and the weight it draws at.
fn face_name(font: &str, css: u16) -> String {
    format!("{font} {css}")
}

/// Every font egui may draw with. The desktop's faces go first in the
/// family of their step, the bundled Inter behind them.
#[must_use = "the definitions do nothing until set on the context"]
pub fn definitions(ui: &UiFaces, fixed: Option<&FontFile>) -> FontDefinitions {
    let mut fonts = FontDefinitions::empty();
    let inter = include_bytes!("../assets/fonts/InterVariable.ttf");
    let steps = [
        (Weight::Regular, ui.regular.as_ref()),
        (Weight::Bold, ui.bold.as_ref()),
    ];

    for (weight, _) in steps {
        let data = weighted(FontData::from_static(inter), weight.css());
        fonts
            .font_data
            .insert(face_name(INTER, weight.css()), Arc::new(data));
    }

    for (weight, desktop) in steps {
        let bundled = face_name(INTER, weight.css());
        let mut family = vec![bundled.clone()];
        if let Some(file) = desktop {
            let key = face_name(&file.name, file.weight);
            if key != bundled {
                fonts
                    .font_data
                    .entry(key.clone())
                    .or_insert_with(|| Arc::new(desktop_data(file)));
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

    let mut monospace = vec![
        JETBRAINS_MONO.to_owned(),
        face_name(INTER, Weight::Regular.css()),
    ];

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
    let key = face_name(&file.name, file.weight);
    if fonts.font_data.contains_key(&key) {
        return;
    }

    fonts
        .font_data
        .insert(key.clone(), Arc::new(desktop_data(file)));
    for family in fonts.families.values_mut() {
        family.push(key.clone());
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

/// The monospace family at `scale` of the desktop's monospace size.
pub fn mono(style: &Style, scale: f32) -> FontId {
    let size = style
        .text_styles
        .get(&TextStyle::Monospace)
        .map_or(DEFAULT_POINTS * PIXELS_PER_POINT, |font| font.size);

    FontId::new(size * scale, FontFamily::Monospace)
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
