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
    for scheme in shipped().schemes() {
        let tokens = tokens_of(scheme);
        let name = scheme.id().as_str();
        let at_least = |ink, fills: &[egui::Color32], ratio: f32, what: &str| {
            for fill in fills {
                let reached = contrast(ink, *fill);
                assert!(
                    reached >= ratio,
                    "{name}: {what} {reached:.2}:1 on {fill:?}"
                );
            }
        };

        let surfaces = [tokens.window, tokens.card, tokens.field, tokens.hover];
        at_least(tokens.text, &[tokens.card], 7.0, "text");
        at_least(tokens.text, &surfaces, 4.5, "text");
        at_least(tokens.secondary, &surfaces, 4.5, "secondary");
        at_least(tokens.dim, &[tokens.card], 3.0, "dim");
        at_least(tokens.on_accent, &[tokens.accent], 4.5, "on accent");
        at_least(tokens.on_tint, &[tokens.accent_tint], 4.5, "on tint");
        at_least(tokens.on_error, &[tokens.error], 4.5, "on error");
        at_least(
            tokens.accent_text,
            &[tokens.window, tokens.card],
            4.5,
            "accent text",
        );
        at_least(
            tokens.accent_ring,
            &[tokens.window, tokens.card, tokens.sidebar],
            3.0,
            "ring",
        );
        let under_rim = [tokens.field, tokens.hover, tokens.card, tokens.window];
        at_least(tokens.field_rim, &under_rim, 3.0, "field rim");
        for status in [tokens.success, tokens.warning, tokens.error] {
            at_least(status, &surfaces, 4.5, "status");
        }

        for hue in [Hue::Accent, Hue::Blue, Hue::Green, Hue::Orange] {
            let tile = tokens.tiles.of(hue);
            at_least(tile.ink, &[tile.fill], 3.0, "tile glyph");
        }

        let terminal = Palette {
            view: tokens.terminal,
            ..palette_of(scheme)
        };

        for index in PRINTED {
            let printed = resolve(Color::Indexed(index), &tokens.terminal_ansi, &terminal);
            at_least(printed, &[tokens.terminal], 4.5, "terminal colour");
        }
    }
}
