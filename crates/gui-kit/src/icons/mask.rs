//! Theme glyphs as masks, so the tint alone gives them their colour.

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
