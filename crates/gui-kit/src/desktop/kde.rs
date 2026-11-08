//! `kdeglobals`: the colour scheme, fonts and icon theme of KDE and LXQt.

#[cfg(test)]
mod tests;

use egui::Color32;

use super::{Colors, DesktopTheme, FontSpec, ini};

/// Qt 5 wrote weights on a 0 to 99 scale, Qt 6 writes CSS weights. Each
/// Qt 5 step maps to the CSS weight it stands for in `QFont::Weight`.
const QT5_WEIGHTS: [(u16, u16); 9] = [
    (0, 100),
    (12, 200),
    (25, 300),
    (50, 400),
    (57, 500),
    (63, 600),
    (75, 700),
    (81, 800),
    (87, 900),
];

/// Unknown or broken lines are skipped.
pub fn parse(text: &str) -> DesktopTheme {
    let mut theme = DesktopTheme::default();
    for (group, key, value) in ini::entries(text) {
        match (group, key) {
            ("General", "font") => theme.font = font(value),
            ("General", "fixed") => theme.fixed_font = font(value),
            ("Icons", "Theme") if !value.is_empty() => theme.icon_theme = Some(value.to_owned()),
            ("KDE", "AnimationDurationFactor") => theme.animations = animations(value),
            _ => {
                if let Some(slot) = color_slot(&mut theme.colors, group, key) {
                    *slot = color(value);
                }
            }
        }
    }

    theme
}

fn color_slot<'colors>(
    colors: &'colors mut Colors,
    group: &str,
    key: &str,
) -> Option<&'colors mut Option<Color32>> {
    let slot = match (group, key) {
        ("Colors:Window", "BackgroundNormal") => &mut colors.window,
        ("Colors:Window", "ForegroundNormal") => &mut colors.text,
        ("Colors:View", "BackgroundNormal") => &mut colors.view,
        ("Colors:View", "ForegroundNegative") => &mut colors.error,
        ("Colors:View", "ForegroundNeutral") => &mut colors.warning,
        ("Colors:View", "ForegroundPositive") => &mut colors.success,
        ("Colors:View", "ForegroundLink") => &mut colors.link,
        ("Colors:View", "ForegroundVisited") => &mut colors.visited,
        ("Colors:Selection", "BackgroundNormal") => &mut colors.accent,
        ("Colors:Selection", "ForegroundNormal") => &mut colors.on_accent,
        _ => return None,
    };

    Some(slot)
}

/// `r,g,b` or `r,g,b,a`, each 0 to 255.
#[expect(
    clippy::disallowed_methods,
    reason = "the palette and the desktop parsers are where colours are made"
)]
fn color(value: &str) -> Option<Color32> {
    let mut parts = value.split(',').map(|part| part.trim().parse::<u8>());
    let red = parts.next()?.ok()?;
    let green = parts.next()?.ok()?;
    let blue = parts.next()?.ok()?;
    let alpha = parts.next().transpose().ok()?.unwrap_or(u8::MAX);
    if parts.next().is_some() {
        return None;
    }

    Some(Color32::from_rgba_unmultiplied(red, green, blue, alpha))
}

/// KDE's animation speed. 0 turns animations off, any other factor
/// only stretches them.
fn animations(factor: &str) -> Option<bool> {
    factor
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|factor| factor.is_finite())
        .map(|factor| factor > 0.0)
}

/// Qt's `QFont::toString`: family, point size, pixel size, style hint,
/// weight, then flags the app does not use.
fn font(value: &str) -> Option<FontSpec> {
    let mut parts = value.split(',').map(str::trim);
    let family = parts.next().filter(|family| !family.is_empty())?;
    let points = parts
        .next()?
        .parse::<f32>()
        .ok()
        .filter(|points| *points > 0.0)?;

    let weight = parts
        .nth(2)
        .and_then(|weight| weight.parse::<u16>().ok())
        .map_or(400, css_weight);

    Some(FontSpec {
        family: family.to_owned(),
        points,
        weight,
    })
}

fn css_weight(qt: u16) -> u16 {
    if qt >= 100 {
        return qt.min(900);
    }

    QT5_WEIGHTS
        .iter()
        .min_by_key(|(step, _)| step.abs_diff(qt))
        .map_or(400, |(_, css)| *css)
}
