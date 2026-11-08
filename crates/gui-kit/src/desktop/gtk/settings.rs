//! GTK's `settings.ini`: the theme whose `gtk.css` holds the colours, the
//! font and the icon theme.

#[cfg(test)]
mod tests;

use crate::desktop::{FontSpec, ini};

/// Pango's style words by CSS weight, written without `-` and lower case.
const WEIGHTS: [(&str, u16); 15] = [
    ("thin", 100),
    ("ultralight", 200),
    ("extralight", 200),
    ("light", 300),
    ("book", 400),
    ("regular", 400),
    ("normal", 400),
    ("medium", 500),
    ("semibold", 600),
    ("demibold", 600),
    ("bold", 700),
    ("ultrabold", 800),
    ("extrabold", 800),
    ("heavy", 900),
    ("black", 900),
];
/// Pango's style words that say nothing about the weight.
const SHAPES: [&str; 6] = [
    "italic",
    "oblique",
    "condensed",
    "semicondensed",
    "expanded",
    "smallcaps",
];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Settings {
    pub theme: Option<String>,
    pub font: Option<FontSpec>,
    pub icon_theme: Option<String>,
    pub prefer_dark: Option<bool>,
    pub enable_animations: Option<bool>,
}

pub fn parse(text: &str) -> Settings {
    let mut settings = Settings::default();
    for (group, key, value) in ini::entries(text) {
        let named = (!value.is_empty()).then(|| value.to_owned());
        match (group, key) {
            ("Settings", "gtk-theme-name") => settings.theme = named,
            ("Settings", "gtk-icon-theme-name") => settings.icon_theme = named,
            ("Settings", "gtk-font-name") => settings.font = pango_font(value),
            ("Settings", "gtk-application-prefer-dark-theme") => {
                settings.prefer_dark = flag(value);
            }
            ("Settings", "gtk-enable-animations") => settings.enable_animations = flag(value),
            _ => {}
        }
    }

    settings
}

/// A GTK boolean, which settings.ini writes as a word or a digit.
fn flag(value: &str) -> Option<bool> {
    match value {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

/// A Pango font description: `Family, Styles Size` or `Family Styles Size`.
fn pango_font(value: &str) -> Option<FontSpec> {
    let (rest, size) = value.trim().rsplit_once(char::is_whitespace)?;
    let points = size.parse::<f32>().ok().filter(|points| *points > 0.0)?;
    let (family, styles) = rest.split_once(',').map_or_else(
        || split_trailing_styles(rest),
        |(family, styles)| {
            (
                family.trim().to_owned(),
                styles.split_whitespace().collect(),
            )
        },
    );

    if family.is_empty() {
        return None;
    }

    let weight = styles
        .iter()
        .rev()
        .find_map(|word| weight_of(word))
        .unwrap_or(400);

    Some(FontSpec {
        family,
        points,
        weight,
    })
}

/// Without a comma the style words are the ones at the end.
fn split_trailing_styles(rest: &str) -> (String, Vec<&str>) {
    let mut words: Vec<&str> = rest.split_whitespace().collect();
    let family_words = words
        .iter()
        .rposition(|word| !is_style(word))
        .map_or(0, |last| last.saturating_add(1));

    let styles = words.split_off(family_words);
    (words.join(" "), styles)
}

fn normalized(word: &str) -> String {
    word.to_ascii_lowercase().replace('-', "")
}

fn weight_of(word: &str) -> Option<u16> {
    let word = normalized(word);
    WEIGHTS
        .iter()
        .find(|(name, _)| *name == word)
        .map(|(_, weight)| *weight)
}

fn is_style(word: &str) -> bool {
    weight_of(word).is_some() || SHAPES.contains(&normalized(word).as_str())
}
