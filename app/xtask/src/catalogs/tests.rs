use std::fs;
use std::path::Path;

use toolbox_catalog::APPS;

use super::{SOURCE_DIR, TOGGLE_FILES, load};
use crate::FIXED_CATALOGS;

const OTHER_FILES: [&str; FIXED_CATALOGS] = [
    "skel.toml",
    toolbox_catalog::CONFIG_FILES,
    toolbox_catalog::COLOUR_SCHEMES,
];

const ENTRY: &str = "[[section]]\nid = \"s\"\ntitle = \"S\"\nicon = \"x\"\n\n[[section.entry]]\nid = \"a\"\ntitle = \"A\"\ndescription = \"D\"\ncheck = { package = \"a\" }\n";

#[test]
fn the_source_catalogs_are_valid() {
    let source = crate::root().join(SOURCE_DIR);
    let catalogs = load(&source).unwrap();

    assert_eq!(catalogs.toggles.len(), TOGGLE_FILES.len() + APPS.len());
    assert_ne!(catalogs.config_files.sections(), []);
    assert!(
        super::unused_writable(&source, &catalogs.config_files).is_empty(),
        "the write table has lines no entry needs"
    );
}

#[test]
fn load_reports_every_broken_catalog() {
    let dir = tempfile::tempdir().unwrap();

    fs::write(dir.path().join("skel.toml"), "[[config]\n").unwrap();

    let errors = load(dir.path()).unwrap_err();
    let messages: Vec<String> = errors.iter().map(ToString::to_string).collect();

    // Skel first, then the toggle catalogs, the apps, the config files
    // and the colour schemes.
    let expected: Vec<&str> = std::iter::once(OTHER_FILES[0])
        .chain(TOGGLE_FILES)
        .chain(APPS)
        .chain(OTHER_FILES.into_iter().skip(1))
        .collect();

    assert_eq!(messages.len(), expected.len(), "{messages:?}");
    for (message, file) in messages.iter().zip(expected) {
        assert!(message.contains(file), "{file}: {messages:?}");
    }
}

#[test]
fn an_icon_missing_from_the_catalogs_icon_folder_is_reported() {
    let source = crate::root().join(SOURCE_DIR);
    let dir = tempfile::tempdir().unwrap();

    copy_catalogs(&source, dir.path(), "gaming.toml");

    fs::write(
        dir.path().join("gaming.toml"),
        format!("{ENTRY}icon = \"a.png\"\n"),
    )
    .unwrap();

    let messages: Vec<String> = load(dir.path())
        .unwrap_err()
        .iter()
        .map(ToString::to_string)
        .collect();

    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(messages[0].contains("gaming.toml"), "{messages:?}");
    assert!(messages[0].contains("\"a.png\" is missing"), "{messages:?}");

    fs::create_dir_all(dir.path().join("icons/gaming")).unwrap();
    fs::write(dir.path().join("icons/gaming/a.png"), "").unwrap();

    assert_eq!(
        load(dir.path()).unwrap().toggles.len(),
        TOGGLE_FILES.len() + APPS.len()
    );
}

#[test]
fn an_app_id_in_two_categories_is_reported() {
    let source = crate::root().join(SOURCE_DIR);
    let dir = tempfile::tempdir().unwrap();

    fs::write(dir.path().join("gaming.toml"), ENTRY).unwrap();

    copy_catalogs(&source, dir.path(), "gaming.toml");

    let documents = fs::read_to_string(source.join("apps/documents.toml")).unwrap();
    let firefox = "\n[[section.entry]]\nid = \"firefox\"\ntitle = \"F\"\ndescription = \"D\"\ncheck = { package = \"firefox\" }\n";

    fs::write(
        dir.path().join("apps/documents.toml"),
        format!("{documents}{firefox}"),
    )
    .unwrap();

    let messages: Vec<String> = load(dir.path())
        .unwrap_err()
        .iter()
        .map(ToString::to_string)
        .collect();

    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(messages[0].contains("internet.toml"), "{messages:?}");
    assert!(
        messages[0].contains("\"firefox\": the id is used twice"),
        "{messages:?}"
    );
}

/// `toggle on <id>` looks over every page, so an id names one switch.
#[test]
fn a_switch_id_on_two_pages_is_reported() {
    let source = crate::root().join(SOURCE_DIR);
    let dir = tempfile::tempdir().unwrap();

    copy_catalogs(&source, dir.path(), "gaming.toml");
    fs::write(
        dir.path().join("gaming.toml"),
        ENTRY.replace("id = \"a\"", "id = \"pipewire-support\""),
    )
    .unwrap();

    let messages: Vec<String> = load(dir.path())
        .unwrap_err()
        .iter()
        .map(ToString::to_string)
        .collect();

    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(messages[0].contains("gaming.toml"), "{messages:?}");
    assert!(
        messages[0].contains("\"pipewire-support\": the id is used twice"),
        "{messages:?}"
    );
}

/// The helper's write table may not keep a folder the page stopped
/// offering.
#[test]
fn a_write_table_line_no_entry_needs_is_reported() {
    let source = crate::root().join(SOURCE_DIR);
    let dir = tempfile::tempdir().unwrap();

    copy_catalogs(&source, dir.path(), "config-files.toml");

    if source.join("icons").is_dir() {
        copy_tree(&source.join("icons"), &dir.path().join("icons"));
    }

    let configs = fs::read_to_string(source.join("config-files.toml")).unwrap();

    fs::write(
        dir.path().join("config-files.toml"),
        configs.replacen(
            "path = \"/etc/sddm.conf.d\"",
            "path = \"~/.config/sddm-test\"",
            1,
        ),
    )
    .unwrap();

    let loaded = load(dir.path()).unwrap();
    let messages: Vec<String> = super::unused_writable(dir.path(), &loaded.config_files)
        .iter()
        .map(ToString::to_string)
        .collect();

    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(messages[0].contains("config-files.toml"), "{messages:?}");
    assert!(messages[0].contains("\"/etc/sddm.conf.d\""), "{messages:?}");
}

#[test]
fn an_entry_that_brings_a_protected_package_along_is_reported() {
    let source = crate::root().join(SOURCE_DIR);
    let dir = tempfile::tempdir().unwrap();

    copy_catalogs(&source, dir.path(), "gaming.toml");
    fs::write(
        dir.path().join("gaming.toml"),
        format!("{ENTRY}also = [{{ package = \"sudo\" }}]\n"),
    )
    .unwrap();

    let messages: Vec<String> = load(dir.path())
        .unwrap_err()
        .iter()
        .map(ToString::to_string)
        .collect();

    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(messages[0].contains("gaming.toml"), "{messages:?}");
    assert!(
        messages[0].contains("\"sudo\" is protected"),
        "{messages:?}"
    );
}

fn copy_catalogs(source: &Path, dir: &Path, left_out: &str) {
    fs::create_dir_all(dir.join("apps")).unwrap();

    for file in TOGGLE_FILES
        .into_iter()
        .chain(OTHER_FILES)
        .chain(APPS)
        .filter(|file| *file != left_out)
    {
        fs::copy(source.join(file), dir.join(file)).unwrap();
    }
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();

    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());

        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}
