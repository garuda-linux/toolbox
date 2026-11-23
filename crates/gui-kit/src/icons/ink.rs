//! The text ink of a colour icon: its `ColorScheme-Text` parts and its
//! `currentColor` parts take the colour of the place the icon is drawn in,
//! as KDE recolours them.

use egui::{Color32, Image};

/// The class whose colour is the text ink.
const TEXT_CLASS: &str = "ColorScheme-Text";

/// A colour icon drawn with `ink` in its text parts. The URI carries the
/// ink, so each colour gets its own texture.
pub(super) fn inked_image(stem: &str, svg: &[u8], ink: Color32) -> Image<'static> {
    inked(&String::from_utf8_lossy(svg), ink).map_or_else(
        || Image::from_bytes(format!("{stem}.svg"), svg.to_vec()),
        |text| {
            let uri = format!("{stem}-{:02x}{:02x}{:02x}.svg", ink.r(), ink.g(), ink.b());
            Image::from_bytes(uri, text.into_bytes())
        },
    )
}

/// The `#rrggbb` form of a colour, for SVG colours.
fn hex(colour: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", colour.r(), colour.g(), colour.b())
}

/// The file's SVG with its text parts set to `ink`: the `ColorScheme-Text`
/// rule's colour and each `currentColor` part without another class. A part
/// of another class (a status icon's coloured body) keeps the colour that
/// class sets. `None` when the file has no text part.
pub(super) fn inked(text: &str, ink: Color32) -> Option<String> {
    if !text.contains("currentColor") && !text.contains(TEXT_CLASS) {
        return None;
    }

    let ink_hex = hex(ink);
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for (start, _) in text.match_indices('<') {
        let Some(close) = text.get(start..).and_then(|rest| rest.find('>')) else {
            break;
        };

        let end = start + close + 1;

        let (Some(before), Some(tag)) = (text.get(last..start), text.get(start..end)) else {
            break;
        };

        if tag.contains("currentColor") {
            let colour = match class_name(tag) {
                Some(class) if class != TEXT_CLASS => {
                    rule_colour(text, class).unwrap_or(ink_hex.as_str())
                }
                _ => ink_hex.as_str(),
            };

            out.push_str(before);
            out.push_str(&tag.replace("currentColor", colour));
            last = end;
        }
    }

    out.push_str(text.get(last..).unwrap_or_default());
    Some(set_text_rule(&out, &ink_hex))
}

/// The first class the element names.
fn class_name(tag: &str) -> Option<&str> {
    tag.split("class=\"").nth(1)?.split('"').next()
}

/// The colour the file's style sets for `class`.
fn rule_colour<'a>(text: &'a str, class: &str) -> Option<&'a str> {
    let rule = text.find(&format!(".{class} {{"))?;
    let body = text.get(rule..)?;
    let value = body.split("color:").nth(1)?.split(';').next()?;

    Some(value.trim())
}

/// Sets the colour of the `ColorScheme-Text` rule to `hex`.
fn set_text_rule(svg: &str, hex: &str) -> String {
    let Some(rule) = svg.find(&format!(".{TEXT_CLASS} {{")) else {
        return svg.to_owned();
    };

    let Some(value) = svg
        .get(rule..)
        .and_then(|rest| rest.find("color:"))
        .map(|at| rule + at + "color:".len())
    else {
        return svg.to_owned();
    };

    let Some(end) = svg
        .get(value..)
        .and_then(|rest| rest.find(';'))
        .map(|at| value + at)
    else {
        return svg.to_owned();
    };

    format!(
        "{} {hex}{}",
        svg.get(..value).unwrap_or_default(),
        svg.get(end..).unwrap_or_default()
    )
}
