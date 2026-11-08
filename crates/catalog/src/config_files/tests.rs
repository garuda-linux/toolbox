mod read_only;

use std::path::Path;

use toolbox_plan::{
    CONFIG_ROOTS, ConfigEntryLocation, WRITABLE_CONFIG_FILES, WRITABLE_CONFIG_FOLDERS,
};

use super::ConfigFileCatalog;
use crate::{CatalogError, Problem};

const SOURCE: &str = "test.toml";

const SMALL: &str = r#"
[[section]]
id = "bootloader"
title = "Bootloader"
icon = "system-reboot"

[[section.entry]]
id = "grub-default"
name = "GRUB default"
path = "/etc/default/grub"
description = "GRUB configuration variables"
url = "https://wiki.archlinux.org/title/GRUB"

[[section]]
id = "user"
title = "User environment"
icon = "utilities-terminal"

[[section.entry]]
id = "bash"
name = "Bash"
path = "~/.bashrc"

[[section.entry]]
id = "user-units"
name = "User systemd units"
path = "~/.config/systemd/user"
directory = true
"#;

fn shipped() -> ConfigFileCatalog {
    ConfigFileCatalog::load(&crate::source_dir()).unwrap()
}

fn problem(text: &str) -> (String, Problem) {
    match ConfigFileCatalog::parse(text, SOURCE).unwrap_err() {
        CatalogError::Invalid {
            source_name,
            id,
            problem,
        } => {
            assert_eq!(source_name, SOURCE);
            (id, problem)
        }
        other @ (CatalogError::Read { path: _, error: _ }
        | CatalogError::Syntax {
            source_name: _,
            message: _,
        }) => panic!("not an invalid entry: {other}"),
    }
}

fn changed(from: &str, to: &str) -> String {
    assert!(SMALL.contains(from), "{from} is not in the catalog");
    SMALL.replacen(from, to, 1)
}

#[test]
fn sections_and_entries_keep_their_order_and_fields() {
    let catalog = ConfigFileCatalog::parse(SMALL, SOURCE).unwrap();
    let sections = catalog.sections();

    assert_eq!(
        sections
            .iter()
            .map(|section| (section.id(), section.title(), section.icon()))
            .collect::<Vec<_>>(),
        [
            ("bootloader", "Bootloader", "system-reboot"),
            ("user", "User environment", "utilities-terminal")
        ]
    );

    let grub = catalog.entry("grub-default").unwrap();

    assert_eq!(grub.name(), "GRUB default");
    assert_eq!(grub.description(), Some("GRUB configuration variables"));
    assert_eq!(grub.url(), Some("https://wiki.archlinux.org/title/GRUB"));
    assert!(!grub.is_directory());
    assert!(
        matches!(grub.location(), ConfigEntryLocation::Root(path) if path.as_str() == "/etc/default/grub")
    );

    let bash = catalog.entry("bash").unwrap();

    assert_eq!((bash.description(), bash.url()), (None, None));
    assert!(
        matches!(bash.location(), ConfigEntryLocation::Home(path) if path.as_str() == ".bashrc")
    );
    assert!(catalog.entry("user-units").unwrap().is_directory());
    assert_eq!(catalog.entry("nothing"), None);
}

#[test]
fn an_empty_file_is_an_empty_catalog() {
    assert_eq!(
        ConfigFileCatalog::parse("", SOURCE).unwrap(),
        ConfigFileCatalog::default()
    );
}

#[test]
fn unknown_fields_and_broken_toml_are_syntax_errors() {
    for text in [
        changed("directory = true", "folder = true"),
        changed("icon = \"system-reboot\"", "colour = \"red\""),
        changed("name = \"Bash\"", ""),
        "[[section".to_owned(),
    ] {
        assert!(matches!(
            ConfigFileCatalog::parse(&text, SOURCE).unwrap_err(),
            CatalogError::Syntax {
                source_name: _,
                message: _
            }
        ));
    }
}

#[test]
fn ids_are_plain_and_unique_over_sections_and_entries() {
    assert_eq!(
        problem(&changed("id = \"bash\"", "id = \"Bash\"")),
        ("Bash".to_owned(), Problem::BadId)
    );
    assert_eq!(
        problem(&changed("id = \"bash\"", "id = \"grub-default\"")),
        ("grub-default".to_owned(), Problem::DuplicateId)
    );
    assert_eq!(
        problem(&changed("id = \"bash\"", "id = \"bootloader\"")),
        ("bootloader".to_owned(), Problem::DuplicateId)
    );
    assert_eq!(
        problem(&changed("id = \"user\"", "id = \"bootloader\"")),
        ("bootloader".to_owned(), Problem::DuplicateId)
    );
}

#[test]
fn texts_must_say_something() {
    assert_eq!(
        problem(&changed("title = \"Bootloader\"", "title = \" \"")).1,
        Problem::EmptyTitle
    );
    assert_eq!(
        problem(&changed("icon = \"system-reboot\"", "icon = \"\"")).1,
        Problem::EmptyIcon
    );
    assert_eq!(
        problem(&changed("name = \"Bash\"", "name = \"\"")).1,
        Problem::EmptyName
    );
    assert_eq!(
        problem(&changed(
            "description = \"GRUB configuration variables\"",
            "description = \"  \""
        ))
        .1,
        Problem::EmptyDescription
    );
    assert_eq!(
        problem("[[section]]\nid = \"empty\"\ntitle = \"Empty\"\nicon = \"folder\"\n"),
        ("empty".to_owned(), Problem::NoEntries)
    );
}

#[test]
fn links_are_https_only() {
    for url in [
        "http://wiki.archlinux.org/title/GRUB",
        "file:///etc/passwd",
        "javascript:alert(1)",
        "https://",
    ] {
        assert_eq!(
            problem(&changed("https://wiki.archlinux.org/title/GRUB", url)),
            ("grub-default".to_owned(), Problem::BadUrl(url.to_owned()))
        );
    }
}

#[test]
fn home_paths_stay_below_the_home() {
    for path in ["~/../etc/passwd", "~/.config/../../x", "~/", "~/a//b"] {
        let (id, found) = problem(&changed("~/.bashrc", path));

        assert_eq!(id, "bash");
        assert!(matches!(found, Problem::BadHomePath(_)), "{path}: {found}");
    }
}

/// The helper trusts a path only below its roots, so an entry elsewhere
/// could never be edited. The catalog refuses it instead of showing it.
#[test]
fn root_paths_must_fit_the_helpers_allowlist() {
    for path in [
        "/home/user/.bashrc",
        "/etc/../root/.bashrc",
        "/usr/lib/sddm/sddm.conf.d/default.conf",
        "/boot/loader/loader.conf",
        "/var/lib/x",
        "/etc/shadow",
        "/etc/gshadow",
        "etc/fstab",
        ".bashrc",
        "~user/.bashrc",
        "",
    ] {
        let (id, found) = problem(&changed("/etc/default/grub", path));

        assert_eq!(id, "grub-default");
        assert!(
            matches!(found, Problem::BadConfigPath(_)),
            "{path}: {found}"
        );
    }
}

#[test]
fn two_entries_must_not_point_at_the_same_path() {
    let (id, found) = problem(&changed(
        "path = \"~/.bashrc\"",
        "path = \"~/.config/systemd/user\"",
    ));

    assert_eq!(id, "user-units");

    assert!(
        matches!(found, Problem::DuplicatePath(_)),
        "expected a duplicate path, got {found:?}"
    );
}

#[test]
fn the_shipped_catalog_is_valid_and_complete() {
    let catalog = shipped();
    let entries: Vec<_> = catalog.entries().collect();

    assert!(catalog.sections().len() >= 13);
    assert!(entries.len() >= 111);
    assert!(entries.iter().filter(|entry| entry.is_directory()).count() >= 16);
    assert!(entries.iter().all(|entry| entry.description().is_some()));
    assert!(entries.iter().all(|entry| entry.url().is_some()));
}

/// A root would be dead weight in the allowlist without an entry below
/// it.
#[test]
fn the_allowlist_is_exactly_what_the_shipped_catalog_needs() {
    let catalog = shipped();
    let roots: Vec<&str> = catalog
        .entries()
        .filter_map(|entry| match entry.location() {
            ConfigEntryLocation::Root(path) => Some(path.as_str()),
            ConfigEntryLocation::Home(_) => None,
        })
        .collect();

    assert!(roots.len() >= 67);

    for root in CONFIG_ROOTS {
        assert!(
            roots.iter().any(|path| path.starts_with(root)),
            "no entry below {root}"
        );
    }
}

/// The helper writes only what its table names, so an entry outside of it
/// could be read but never saved.
#[test]
fn root_entries_must_be_in_the_helpers_write_table() {
    for path in [
        "/etc/ld.so.preload\"",
        "/etc/sudoers.d\"\ndirectory = true",
        "/etc/pam.d\"",
        "/etc/fstab\"\ndirectory = true",
    ] {
        let (id, found) = problem(&changed("/etc/default/grub\"", path));

        assert_eq!(id, "grub-default");
        assert!(matches!(found, Problem::NotWritable(_)), "{path}: {found}");
    }
}

/// A line of the table no entry needs would let the helper write a file
/// the page never offers.
#[test]
fn the_write_table_is_exactly_what_the_shipped_catalog_needs() {
    assert_eq!(shipped().unused_writable(), Vec::<&str>::new());

    let small = ConfigFileCatalog::parse(SMALL, SOURCE).unwrap();
    let unused = small.unused_writable();

    assert_eq!(
        unused.len(),
        WRITABLE_CONFIG_FILES.len() + WRITABLE_CONFIG_FOLDERS.len() - 1
    );
    assert!(!unused.contains(&"/etc/default/grub"));
    assert!(unused.contains(&"/etc/modprobe.d"));
}

/// The self-test reads its own catalog in place of the shipped one, under
/// the same rules: its edit must reach a file the helper writes.
#[test]
fn the_catalog_of_the_self_test_loads_under_the_same_rules() {
    let catalog = ConfigFileCatalog::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/selftest/configs"),
    )
    .unwrap();

    for (id, read_only) in [
        ("fstab", false),
        ("modprobe-d", false),
        ("bash", false),
        ("passwd", true),
        ("sudoers", true),
    ] {
        assert_eq!(catalog.entry(id).unwrap().is_read_only(), read_only, "{id}");
    }
}
