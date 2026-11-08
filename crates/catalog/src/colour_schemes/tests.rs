use toolbox_plan::{InvalidRgb, Rgb, SchemeId};

use super::{ColourSchemeCatalog, SchemeMode};
use crate::{CatalogError, Problem};

const DARK: [&str; 15] = [
    "catppuccin-frappe",
    "catppuccin-macchiato",
    "catppuccin-mocha",
    "nord",
    "dracula",
    "sweet",
    "tokyo-night",
    "gruvbox",
    "solarized-dark",
    "rose-pine",
    "rose-pine-moon",
    "everforest",
    "one-dark",
    "ayu-dark",
    "ayu-mirage",
];

const LIGHT: [&str; 9] = [
    "catppuccin-latte",
    "tokyo-night-day",
    "nord-light",
    "dracula-alucard",
    "gruvbox-light",
    "solarized-light",
    "rose-pine-dawn",
    "everforest-light",
    "ayu-light",
];

const WITHOUT_ANSI: [&str; 12] = [
    "nord-light",
    "dracula-alucard",
    "gruvbox-light",
    "rose-pine",
    "rose-pine-moon",
    "rose-pine-dawn",
    "everforest",
    "everforest-light",
    "one-dark",
    "ayu-dark",
    "ayu-mirage",
    "ayu-light",
];

const ROLES: &str = "window = \"#1e1e2e\"\nview = \"#181825\"\ntext = \"#cdd6f4\"\n\
    accent = \"#cba6f7\"\non_accent = \"#1e1e2e\"\nerror = \"#f38ba8\"\n\
    warning = \"#f9e2af\"\nsuccess = \"#a6e3a1\"\nlink = \"#89b4fa\"\n\
    visited = \"#b4befe\"\n\
    tiles = { blue = \"#89b4fa\", green = \"#a6e3a1\", orange = \"#fab387\" }\n";

fn one(head: &str, extra: &str) -> String {
    format!("[[scheme]]\n{head}{ROLES}{extra}")
}

fn mocha(extra: &str) -> String {
    one(
        "id = \"mocha\"\nname = \"Mocha\"\nmode = \"dark\"\n\
         source = \"https://github.com/catppuccin/palette\"\nlicence = \"MIT\"\n",
        extra,
    )
}

fn problem_of(text: &str) -> (String, Problem) {
    match ColourSchemeCatalog::parse(text, "test.toml") {
        Err(CatalogError::Invalid {
            source_name: _,
            id,
            problem,
        }) => (id, problem),
        other => panic!("expected a validation error, got {other:?}"),
    }
}

fn shipped() -> ColourSchemeCatalog {
    ColourSchemeCatalog::load(&crate::source_dir()).unwrap()
}

#[test]
fn the_shipped_schemes_are_the_owners_lists_in_order() {
    let catalog = shipped();
    let ids = |mode| {
        catalog
            .of(mode)
            .map(|scheme| scheme.id().as_str().to_owned())
            .collect::<Vec<_>>()
    };

    assert_eq!(ids(SchemeMode::Dark), DARK);
    assert_eq!(ids(SchemeMode::Light), LIGHT);
}

#[test]
fn every_shipped_scheme_names_its_source_and_licence() {
    for scheme in shipped().schemes() {
        assert!(crate::is_web_url(scheme.source()), "{:?}", scheme.id());
        assert!(!scheme.licence().trim().is_empty(), "{:?}", scheme.id());
        assert!(!scheme.name().trim().is_empty(), "{:?}", scheme.id());
    }
}

#[test]
fn the_owners_overrides_hold() {
    let catalog = shipped();
    let colours = |id: &str, mode| {
        let id = SchemeId::new(id).unwrap();
        *catalog.find(&id, mode).unwrap().colours()
    };

    let nord = colours("nord", SchemeMode::Dark);
    assert_eq!(nord.window, Rgb::parse("#3b4252").unwrap());
    assert_eq!(nord.view, Rgb::parse("#2e3440").unwrap());
    assert_eq!(nord.accent, Rgb::parse("#88c0d0").unwrap());

    let light = colours("nord-light", SchemeMode::Light);
    assert_eq!(light.window, Rgb::parse("#eceff4").unwrap());
    assert_eq!(light.accent, Rgb::parse("#5e81ac").unwrap());

    let tokyo = colours("tokyo-night", SchemeMode::Dark);
    assert_eq!(tokyo.accent, Rgb::parse("#7aa2f7").unwrap());

    let mocha = colours("catppuccin-mocha", SchemeMode::Dark);
    assert_eq!(mocha.on_accent, mocha.window, "on accent is base");
}

#[test]
fn the_schemes_without_terminal_colours_of_their_own_have_none() {
    let catalog = shipped();
    for scheme in catalog.schemes() {
        let own = !WITHOUT_ANSI.contains(&scheme.id().as_str());
        assert_eq!(scheme.ansi().is_some(), own, "{:?}", scheme.id());
    }
}

#[test]
fn a_scheme_is_found_only_in_its_own_mode() {
    let catalog = ColourSchemeCatalog::parse(&mocha(""), "test.toml").unwrap();
    let id = SchemeId::new("mocha").unwrap();

    assert!(catalog.find(&id, SchemeMode::Dark).is_some());
    assert!(catalog.find(&id, SchemeMode::Light).is_none());
    assert!(
        catalog
            .find(&SchemeId::new("latte").unwrap(), SchemeMode::Dark)
            .is_none()
    );
}

#[test]
fn ansi_colours_keep_their_order() {
    let list = (0_u8..16)
        .map(|index| format!("\"#0000{index:02x}\""))
        .collect::<Vec<_>>()
        .join(", ");

    let catalog =
        ColourSchemeCatalog::parse(&mocha(&format!("ansi = [{list}]\n")), "test.toml").unwrap();

    let ansi = catalog.schemes()[0].ansi().unwrap();

    assert_eq!(ansi[0].bytes(), [0, 0, 0]);
    assert_eq!(ansi[15].bytes(), [0, 0, 15]);
}

#[test]
fn a_bad_colour_names_the_text() {
    let text = mocha("").replace("\"#cba6f7\"", "\"purple\"");

    assert_eq!(
        problem_of(&text),
        (
            "mocha".to_owned(),
            Problem::BadColour(InvalidRgb("purple".to_owned()))
        )
    );
}

#[test]
fn the_terminal_needs_all_sixteen_colours() {
    let text = mocha("ansi = [\"#000000\", \"#ffffff\"]\n");

    assert_eq!(
        problem_of(&text),
        ("mocha".to_owned(), Problem::AnsiCount(2))
    );
}

#[test]
fn each_entry_needs_an_id_a_name_a_source_and_a_licence() {
    let bad_id = one(
        "id = \"Mocha\"\nname = \"Mocha\"\nmode = \"dark\"\n\
         source = \"https://example.org\"\nlicence = \"MIT\"\n",
        "",
    );

    assert_eq!(problem_of(&bad_id).1, Problem::BadId);

    let unnamed = mocha("").replace("name = \"Mocha\"", "name = \" \"");
    assert_eq!(problem_of(&unnamed).1, Problem::EmptyName);

    let plain = mocha("").replace(
        "https://github.com/catppuccin/palette",
        "http://example.org",
    );

    assert_eq!(
        problem_of(&plain).1,
        Problem::BadUrl("http://example.org".to_owned())
    );

    let unlicensed = mocha("").replace("licence = \"MIT\"", "licence = \"\"");
    assert_eq!(problem_of(&unlicensed).1, Problem::EmptyLicence);

    let twice = format!("{}{}", mocha(""), mocha(""));
    assert_eq!(problem_of(&twice).1, Problem::DuplicateId);
}

#[test]
fn an_unknown_mode_or_field_is_a_syntax_error() {
    let dusk = mocha("").replace("mode = \"dark\"", "mode = \"dusk\"");
    let extra = mocha("shadow = \"#000000\"\n");

    for text in [dusk, extra] {
        assert!(
            matches!(
                ColourSchemeCatalog::parse(&text, "test.toml"),
                Err(CatalogError::Syntax {
                    source_name: _,
                    message: _
                })
            ),
            "{text}"
        );
    }
}
