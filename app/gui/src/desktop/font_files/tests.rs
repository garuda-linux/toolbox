use std::path::Path;

use toolbox_gui_kit::desktop::FontSpec;
use toolbox_gui_kit::fonts;

use super::FontFinder;

/// The bundled file names its family "Inter Variable", not "Inter".
fn finder_with_inter() -> FontFinder {
    let mut database = fontdb::Database::new();
    database
        .load_font_file(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../crates/gui-kit/assets/fonts/InterVariable.ttf"),
        )
        .unwrap();
    FontFinder::from_database(database)
}

fn spec(family: &str, weight: u16) -> FontSpec {
    FontSpec {
        family: family.to_owned(),
        points: 10.0,
        weight,
    }
}

#[test]
fn a_font_the_system_has_is_read_with_its_weight() {
    let found = finder_with_inter()
        .find(&spec("Inter Variable", 700))
        .unwrap();

    assert_eq!(found.name, "Inter Variable");
    assert!(
        fonts::has_weight_axis(&found.data, found.index),
        "the variable face draws the weight"
    );
    assert!(found.data.starts_with(&[0, 1, 0, 0]), "a TrueType file");
}

#[test]
fn a_family_the_system_lacks_is_none() {
    assert_eq!(finder_with_inter().find(&spec("No Such Font", 400)), None);
}

#[test]
fn family_names_match_without_case() {
    assert!(
        finder_with_inter()
            .find(&spec("inter variable", 400))
            .is_some()
    );
}
