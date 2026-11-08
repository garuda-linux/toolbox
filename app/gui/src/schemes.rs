//! The app's own colour schemes as palettes: the one picked for dark and
//! the one picked for light take the desktop's place, whole.

#[cfg(test)]
mod tests;

use toolbox_catalog::{ColourScheme, ColourSchemeCatalog, SchemeMode};
use toolbox_config::Colours;
use toolbox_gui_kit::Palette;
use toolbox_gui_kit::palette::{TileHues, colour};
use toolbox_gui_kit::terminal_colors::Ansi;
use toolbox_plan::{Rgb, SchemeId};

/// `scheme` as a palette, accent, tile hues and terminal colours
/// included, so nothing of the desktop's shows through.
pub fn palette_of(scheme: &ColourScheme) -> Palette {
    let roles = scheme.colours();
    let tiles = scheme.tiles();
    let of = |rgb: Rgb| colour(rgb.bytes());

    Palette {
        window: of(roles.window),
        view: of(roles.view),
        text: of(roles.text),
        accent: of(roles.accent),
        on_accent: of(roles.on_accent),
        error: of(roles.error),
        warning: of(roles.warning),
        success: of(roles.success),
        link: Some(of(roles.link)),
        visited: Some(of(roles.visited)),
        tiles: Some(TileHues {
            blue: of(tiles.blue),
            green: of(tiles.green),
            orange: of(tiles.orange),
        }),
        ansi: scheme.ansi().map(|own| Ansi::new(own.map(of))),
    }
}

/// The dark and light palettes the window draws with.
///
/// Each is the desktop's, replaced by the scheme `colours` picks for its
/// mode. A pick the catalog does not hold for that mode keeps the
/// desktop's, so a removed or mistyped scheme never leaves the window
/// without colours.
pub fn palettes(
    (dark, light): (Palette, Palette),
    catalog: &ColourSchemeCatalog,
    colours: &Colours,
) -> (Palette, Palette) {
    let pick = |id: Option<&SchemeId>, mode, desktop| {
        id.and_then(|id| catalog.find(id, mode))
            .map_or(desktop, palette_of)
    };

    (
        pick(colours.dark_scheme.as_ref(), SchemeMode::Dark, dark),
        pick(colours.light_scheme.as_ref(), SchemeMode::Light, light),
    )
}
