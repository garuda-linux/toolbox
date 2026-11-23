mod letters;

use std::path::Path;

use egui::Theme;
use toolbox_catalog::{ColourScheme, ColourSchemeCatalog, SchemeMode};
use toolbox_config::Colours;
use toolbox_gui_kit::Palette;
use toolbox_gui_kit::palette::colour;
use toolbox_gui_kit::terminal_colors::{PRINTED, resolve};
use toolbox_gui_kit::tokens::{Hue, Tokens, contrast};
use toolbox_plan::SchemeId;
use toolbox_terminal::Color;

use super::{palette_of, palettes};

/// The contrast checks of one scheme, each miss named by its role.
struct Contrast<'a> {
    name: &'a str,
    found: &'a mut Vec<String>,
}

impl Contrast<'_> {
    fn at_least(&mut self, ink: egui::Color32, fills: &[egui::Color32], ratio: f32, what: &str) {
        for fill in fills {
            let reached = contrast(ink, *fill);
            if reached < ratio {
                self.found
                    .push(format!("{}: {what} {reached:.2}:1 on {fill:?}", self.name));
            }
        }
    }
}

fn shipped() -> ColourSchemeCatalog {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog");
    ColourSchemeCatalog::load(&dir).unwrap()
}

fn tokens_of(scheme: &ColourScheme) -> Tokens {
    let theme = match scheme.mode() {
        SchemeMode::Dark => Theme::Dark,
        SchemeMode::Light => Theme::Light,
    };

    Tokens::derive(&palette_of(scheme), theme)
}

fn desktop() -> (Palette, Palette) {
    let mut dark = Palette::fallback(Theme::Dark);
    dark.accent = colour([0x3d, 0xae, 0xe9]);
    let mut light = Palette::fallback(Theme::Light);
    light.accent = colour([0x29, 0x80, 0xb9]);
    (dark, light)
}

fn picked(dark: Option<&str>, light: Option<&str>) -> Colours {
    let id = |name: &str| SchemeId::new(name).unwrap();
    Colours {
        dark_scheme: dark.map(id),
        light_scheme: light.map(id),
    }
}

#[test]
fn a_schemes_palette_takes_every_role_its_tiles_and_its_terminal() {
    let catalog = shipped();
    let nord = catalog
        .find(&SchemeId::new("nord").unwrap(), SchemeMode::Dark)
        .unwrap();

    let palette = palette_of(nord);
    let roles = nord.colours();

    assert_eq!(palette.window, colour(roles.window.bytes()));
    assert_eq!(palette.view, colour(roles.view.bytes()));
    assert_eq!(palette.accent, colour(roles.accent.bytes()));
    assert_eq!(palette.on_accent, colour(roles.on_accent.bytes()));
    assert_eq!(palette.link, Some(colour(roles.link.bytes())));
    assert_eq!(palette.visited, Some(colour(roles.visited.bytes())));
    assert_eq!(
        palette.tiles.map(|tiles| tiles.orange),
        Some(colour(nord.tiles().orange.bytes()))
    );
    assert!(palette.ansi.is_some());
}

#[test]
fn a_scheme_without_terminal_colours_leaves_them_to_the_roles() {
    let catalog = shipped();
    let alucard = catalog
        .find(
            &SchemeId::new("dracula-alucard").unwrap(),
            SchemeMode::Light,
        )
        .unwrap();

    assert_eq!(palette_of(alucard).ansi, None);
}

#[test]
fn system_keeps_the_desktops_palettes_exactly() {
    let desktop = desktop();

    assert_eq!(palettes(desktop, &shipped(), &Colours::default()), desktop);
}

#[test]
fn each_mode_takes_its_own_pick_and_the_other_stays_the_desktops() {
    let desktop = desktop();
    let catalog = shipped();

    let (dark, light) = palettes(desktop, &catalog, &picked(Some("dracula"), None));
    assert_eq!(dark.window, colour([0x28, 0x2a, 0x36]));
    assert_eq!(
        dark.accent,
        colour([0xbd, 0x93, 0xf9]),
        "the scheme's accent"
    );
    assert_eq!(light, desktop.1);

    let (kept, nord) = palettes(desktop, &catalog, &picked(None, Some("nord-light")));
    assert_eq!(kept, desktop.0);
    assert_eq!(nord.accent, colour([0x5e, 0x81, 0xac]));
}

#[test]
fn an_unknown_scheme_or_one_of_the_other_mode_keeps_the_desktops() {
    let desktop = desktop();
    let catalog = shipped();

    let chosen = picked(Some("solarized"), Some("nord"));
    assert_eq!(palettes(desktop, &catalog, &chosen), desktop);
}

/// Official colours, untuned: the tokens fit every miss to its target.
#[test]
fn every_scheme_reaches_the_contrast_targets_after_fitting() {
    let mut shortfalls = Vec::new();
    for scheme in shipped().schemes() {
        check_scheme(scheme, &mut shortfalls);
    }

    assert!(shortfalls.is_empty(), "{shortfalls:#?}");
}

fn check_scheme(scheme: &ColourScheme, found: &mut Vec<String>) {
    let tokens = tokens_of(scheme);

    let name = scheme.id().as_str();
    let mut check = Contrast { name, found };

    let surfaces = [
        tokens.window,
        tokens.card,
        tokens.field,
        tokens.button_hover,
    ];

    check_text(&mut check, &tokens, &surfaces);

    check_accents(&mut check, &tokens);

    // The focus frame is the accent itself, with no 3:1 fit, so the ring
    // is not fitted to the surfaces.
    assert_eq!(tokens.accent_ring, tokens.accent, "{name}: the ring");

    for status in [tokens.success, tokens.warning, tokens.error] {
        check.at_least(status, &surfaces, 4.5, "status");
    }

    for hue in [Hue::Accent, Hue::Blue, Hue::Green, Hue::Orange] {
        let tile = tokens.tiles.of(hue);
        check.at_least(tile.ink, &[tile.fill], 3.0, "tile glyph");
    }

    let terminal = Palette {
        view: tokens.terminal,
        ..palette_of(scheme)
    };

    for index in PRINTED {
        let printed = resolve(Color::Indexed(index), &tokens.terminal_ansi, &terminal);
        check.at_least(printed, &[tokens.terminal], 4.5, "terminal colour");
    }
}

fn check_text(check: &mut Contrast<'_>, tokens: &Tokens, surfaces: &[egui::Color32]) {
    check.at_least(tokens.text, &[tokens.card], 7.0, "text");
    check.at_least(tokens.text, surfaces, 4.5, "text");

    check.at_least(tokens.secondary, surfaces, 4.5, "secondary");
    check.at_least(tokens.dim, &[tokens.card], 3.0, "dim");
}

fn check_accents(check: &mut Contrast<'_>, tokens: &Tokens) {
    check.at_least(tokens.on_accent, &[tokens.accent], 4.5, "on accent");
    check.at_least(tokens.on_tint, &[tokens.accent_tint], 4.5, "on tint");

    check.at_least(tokens.on_error, &[tokens.error], 4.5, "on error");
    check.at_least(
        tokens.accent_text,
        &[tokens.window, tokens.card],
        4.5,
        "accent text",
    );
}
