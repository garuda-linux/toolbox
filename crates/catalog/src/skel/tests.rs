use std::fs;
use std::path::PathBuf;

use super::SkelCatalog;
use crate::{CatalogError, Problem};

const EXPECTED_IDS: [&str; 19] = [
    "bash",
    "fish",
    "zsh",
    "kvantum",
    "libinput",
    "bleachbit",
    "libreoffice",
    "starship",
    "smplayer",
    "vlc",
    "deluge",
    "mpv",
    "micro",
    "bat",
    "environment",
    "gtk",
    "pacseek",
    "psd",
    "kde",
];

fn one(files: &str) -> String {
    format!("[[config]]\nid = \"bash\"\nname = \"Bash\"\nfiles = {files}\n")
}

fn problem_of(text: &str) -> (String, String, Problem) {
    match SkelCatalog::parse(text, "test.toml") {
        Err(CatalogError::Invalid {
            source_name,
            id,
            problem,
        }) => (source_name, id, problem),
        other => panic!("expected a validation error, got {other:?}"),
    }
}

#[test]
fn parses_entries_with_paths_relative_to_skel() {
    let text = one(r#"["/etc/skel/.bashrc_garuda", "/etc/skel/.config/autostart/x.desktop"]"#);
    let catalog = SkelCatalog::parse(&text, "test.toml").unwrap();
    let entry = &catalog.entries()[0];

    assert_eq!(entry.id(), "bash");
    assert_eq!(entry.name(), "Bash");
    assert_eq!(
        entry.files(),
        [
            PathBuf::from(".bashrc_garuda"),
            PathBuf::from(".config/autostart/x.desktop")
        ]
    );
    assert_eq!(catalog.entry("bash"), Some(entry));
    assert_eq!(catalog.entry("fish"), None);
}

#[test]
fn an_empty_file_is_an_empty_catalog() {
    assert_eq!(
        SkelCatalog::parse("", "test.toml").unwrap(),
        SkelCatalog::default()
    );
}

#[test]
fn syntax_errors_and_unknown_fields_name_the_file() {
    for text in [
        "[[config]\n",
        "[[config]]\nid = \"bash\"\nname = \"Bash\"\nfiles = []\nextra = 1\n",
        "[[config]]\nid = \"bash\"\nfiles = [\"/etc/skel/.zshrc\"]\n",
    ] {
        match SkelCatalog::parse(text, "test.toml") {
            Err(
                error @ CatalogError::Syntax {
                    source_name: _,
                    error: _,
                },
            ) => {
                assert!(error.to_string().starts_with("test.toml: "), "{error}");
            }
            other => panic!("expected a syntax error for {text:?}, got {other:?}"),
        }
    }
}

#[test]
fn rejects_bad_ids() {
    for id in ["", "Bash", "bash_x", "-bash", "bash-", "ba--sh", "ba sh"] {
        let text = format!("[[config]]\nid = \"{id}\"\nname = \"B\"\nfiles = [\"/etc/skel/x\"]\n");

        assert_eq!(
            problem_of(&text),
            ("test.toml".to_owned(), id.to_owned(), Problem::BadId)
        );
    }
}

#[test]
fn accepts_digits_in_ids() {
    let text = "[[config]]\nid = \"gtk-4\"\nname = \"G\"\nfiles = [\"/etc/skel/x\"]\n";

    SkelCatalog::parse(text, "test.toml").unwrap();
}

#[test]
fn rejects_duplicate_ids() {
    let text = format!("{}{}", one(r#"["/etc/skel/a"]"#), one(r#"["/etc/skel/b"]"#));

    assert_eq!(problem_of(&text).2, Problem::DuplicateId);
}

#[test]
fn rejects_an_empty_name() {
    let text = "[[config]]\nid = \"bash\"\nname = \" \"\nfiles = [\"/etc/skel/x\"]\n";

    assert_eq!(problem_of(text).2, Problem::EmptyName);
}

#[test]
fn rejects_an_entry_without_files() {
    assert_eq!(problem_of(&one("[]")).2, Problem::NoFiles);
}

#[test]
fn rejects_paths_outside_skel() {
    for path in [
        "/etc/skel",
        "/etc/skel/",
        "/etc/skeleton/x",
        "/home/nico/.bashrc",
        "etc/skel/x",
        "/etc/skel//etc/passwd",
    ] {
        let error = problem_of(&one(&format!("[\"{path}\"]")));

        assert_eq!(error.2, Problem::OutsideSkel(path.to_owned()));
    }
}

#[test]
fn rejects_parent_components() {
    for path in [
        "/etc/skel/../passwd",
        "/etc/skel/.config/../../x",
        "/etc/skel/a/..",
    ] {
        let error = problem_of(&one(&format!("[\"{path}\"]")));

        assert_eq!(error.2, Problem::ParentComponent(path.to_owned()));
    }
}

#[test]
fn validation_errors_name_the_file_and_the_entry() {
    let error = SkelCatalog::parse(&one("[]"), "skel.toml").unwrap_err();

    assert_eq!(error.to_string(), "skel.toml: entry \"bash\": no files");
}

#[test]
fn load_reads_skel_toml_from_the_folder() {
    let dir = tempfile::tempdir().unwrap();

    fs::write(dir.path().join("skel.toml"), one(r#"["/etc/skel/.zshrc"]"#)).unwrap();

    let catalog = SkelCatalog::load(dir.path()).unwrap();

    assert_eq!(catalog.entries().len(), 1);
}

#[test]
fn load_errors_name_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let missing = SkelCatalog::load(dir.path()).unwrap_err();

    assert!(
        matches!(missing, CatalogError::Read { path: _, error: _ }),
        "{missing:?}"
    );
    assert!(missing.to_string().contains("skel.toml"), "{missing}");

    fs::write(dir.path().join("skel.toml"), one("[]")).unwrap();

    let invalid = SkelCatalog::load(dir.path()).unwrap_err();
    let expected = format!(
        "{}: entry \"bash\": no files",
        dir.path().join("skel.toml").display()
    );

    assert_eq!(invalid.to_string(), expected);
}

#[test]
fn the_shipped_catalog_has_its_nineteen_entries() {
    let catalog = SkelCatalog::load(&crate::source_dir()).unwrap();
    let ids: Vec<&str> = catalog.entries().iter().map(super::SkelEntry::id).collect();

    assert_eq!(ids, EXPECTED_IDS);

    let kde = catalog.entry("kde").unwrap();

    assert_eq!(kde.name(), "KDE settings");
    assert_eq!(kde.files().len(), 13);
    assert_eq!(
        kde.files().last(),
        Some(&PathBuf::from(".local/share/kxmlgui5/dolphin/dolphinui.rc"))
    );
    assert_eq!(catalog.entry("gtk").unwrap().name(), "GTK3/4");
    assert_eq!(catalog.entry("psd").unwrap().name(), "Profile sync daemon");
}
