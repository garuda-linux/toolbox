//! `gtk.css`: the `@define-color` names of libadwaita and GTK 3 themes.
//!
//! Only plain values and references to other names resolve. GTK's colour
//! functions (`mix`, `shade`, `alpha`) are skipped, so the role keeps the
//! palette below.

#[cfg(test)]
mod tests;

pub mod settings;

use std::collections::BTreeMap;

use egui::Color32;
use egui::ecolor::linear_u8_from_linear_f32;

use super::Colors;

const DEFINE: &str = "@define-color";
/// How deep `@a` → `@b` → … is followed, against loops.
const MAX_REFERENCE_DEPTH: usize = 8;

pub fn parse(text: &str) -> Colors {
    let css = without_comments(text);
    let defined = definitions(&css);
    let pick = |names: &[&str]| {
        names
            .iter()
            .find_map(|name| resolve(&defined, name, MAX_REFERENCE_DEPTH))
    };

    Colors {
        window: pick(&["window_bg_color", "theme_bg_color"]),
        view: pick(&["view_bg_color", "theme_base_color"]),
        text: pick(&["window_fg_color", "theme_fg_color"]),
        accent: pick(&["accent_bg_color", "theme_selected_bg_color"]),
        on_accent: pick(&["accent_fg_color", "theme_selected_fg_color"]),
        error: pick(&["error_color"]),
        warning: pick(&["warning_color"]),
        success: pick(&["success_color"]),
        link: None,
        visited: None,
    }
}

fn without_comments(text: &str) -> String {
    let mut kept = String::with_capacity(text.len());
    let mut rest = text;
    while let Some((before, after)) = rest.split_once("/*") {
        kept.push_str(before);
        rest = after.split_once("*/").map_or("", |(_, tail)| tail);
    }

    kept.push_str(rest);
    kept
}

fn definitions(css: &str) -> BTreeMap<&str, &str> {
    css.split(DEFINE)
        .skip(1)
        .filter_map(|statement| {
            let (body, _) = statement.split_once(';')?;
            let (name, value) = body.trim().split_once(char::is_whitespace)?;
            Some((name, value.trim()))
        })
        .collect()
}

fn resolve(defined: &BTreeMap<&str, &str>, name: &str, depth: usize) -> Option<Color32> {
    let value = defined.get(name)?;
    match value.strip_prefix('@') {
        Some(other) => resolve(defined, other, depth.checked_sub(1)?),
        None => color(value),
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "the palette and the desktop parsers are where colours are made"
)]
fn color(value: &str) -> Option<Color32> {
    if let Some(hex) = value.strip_prefix('#') {
        return hex_color(hex);
    }

    let (function, arguments) = value.strip_suffix(')')?.split_once('(')?;
    let mut parts = arguments.split(',').map(str::trim);
    let red = parts.next()?.parse::<u8>().ok()?;
    let green = parts.next()?.parse::<u8>().ok()?;
    let blue = parts.next()?.parse::<u8>().ok()?;
    let alpha = match function.trim() {
        "rgb" => u8::MAX,
        "rgba" => linear_u8_from_linear_f32(parts.next()?.parse::<f32>().ok()?.clamp(0.0, 1.0)),
        _ => return None,
    };

    if parts.next().is_some() {
        return None;
    }

    Some(Color32::from_rgba_unmultiplied(red, green, blue, alpha))
}

/// `rgb`, `rrggbb` or `rrggbbaa`.
#[expect(
    clippy::disallowed_methods,
    reason = "the palette and the desktop parsers are where colours are made"
)]
fn hex_color(hex: &str) -> Option<Color32> {
    if !hex.chars().all(|digit| digit.is_ascii_hexdigit()) {
        return None;
    }

    let full = match hex.len() {
        3 => {
            let doubled: String = hex.chars().flat_map(|digit| [digit, digit]).collect();
            format!("{doubled}ff")
        }
        6 => format!("{hex}ff"),
        8 => hex.to_owned(),
        _ => return None,
    };

    let [red, green, blue, alpha] = u32::from_str_radix(&full, 16).ok()?.to_be_bytes();

    Some(Color32::from_rgba_unmultiplied(red, green, blue, alpha))
}
