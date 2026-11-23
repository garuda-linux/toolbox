//! Theme glyphs as masks, so the tint alone gives them their colour.

use std::collections::BTreeSet;

/// `svg` with every colour white: `currentColor` and each `#rgb`,
/// `#rgba`, `#rrggbb` or `#rrggbbaa`, whose alpha digits stay. A `#` that
/// names an element, as in `url(#id)` or `href="#id"`, stays as it is.
pub(super) fn mask(svg: &str) -> String {
    let svg = svg.replace("currentColor", "#ffffff");
    let mut out = String::with_capacity(svg.len());
    let mut chars = svg.chars().peekable();
    while let Some(next) = chars.next() {
        out.push(next);
        if next != '#' || out.ends_with("url(#") || out.ends_with("href=\"#") {
            continue;
        }

        let mut word = String::new();
        while let Some(&part) = chars.peek() {
            if !(part.is_ascii_alphanumeric() || part == '_' || part == '-') {
                break;
            }

            word.push(part);
            chars.next();
        }

        out.push_str(&whiten(&word));
    }

    out
}

/// The white of the colour `word`, with its alpha digits. Any other word
/// stays.
fn whiten(word: &str) -> String {
    if !word.chars().all(|digit| digit.is_ascii_hexdigit()) {
        return word.to_owned();
    }

    match word.len() {
        3 => "fff".to_owned(),
        4 => format!("fff{}", word.get(3..).unwrap_or_default()),
        6 => "ffffff".to_owned(),
        8 => format!("ffffff{}", word.get(6..).unwrap_or_default()),
        _ => word.to_owned(),
    }
}

/// Whether `svg` paints in two fixed colours or more, outside its `<style>`
/// element. Such a file is a colour picture, which [`mask`] would flatten
/// into one blot of the tint. A monochrome file paints in `currentColor`
/// or a class its style colours, so its style may name many colours.
pub(super) fn paints_in_colours(svg: &str) -> bool {
    let mut colours = BTreeSet::new();
    for (index, drawn) in svg.split("<style").enumerate() {
        let drawn = if index == 0 {
            drawn
        } else {
            drawn.split_once("</style>").map_or("", |(_, rest)| rest)
        };

        for (at, _) in drawn.match_indices('#') {
            let (before, after) = drawn.split_at(at);
            if before.ends_with("url(") || before.ends_with("href=\"") {
                continue;
            }

            let hex: String = after
                .chars()
                .skip(1)
                .take_while(char::is_ascii_alphanumeric)
                .collect();

            if matches!(hex.len(), 3 | 4 | 6 | 8)
                && hex.chars().all(|digit| digit.is_ascii_hexdigit())
            {
                colours.insert(hex.to_ascii_lowercase());
            }
        }
    }

    colours.len() > 1
}
