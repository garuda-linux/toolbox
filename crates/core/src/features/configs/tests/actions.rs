//! What a row offers: root-owned files are edited, files of the user open
//! in their app, and what hands out root or the accounts is only viewed.

use std::collections::BTreeSet;

use toolbox_catalog::ConfigFileCatalog;

use super::super::{Found, Link, Listing, RowKind, rows, view_target};
use super::{CATALOG, shown};

const READ_ONLY: &str = r#"
[[section]]
id = "accounts"
title = "Users and authentication"
icon = "system-users"

[[section.entry]]
id = "passwd"
name = "Passwd"
path = "/etc/passwd"
read_only = true

[[section.entry]]
id = "pam"
name = "PAM"
path = "/etc/pam.d"
directory = true
read_only = true
"#;

#[test]
fn rows_say_how_a_file_opens_and_who_owns_it() {
    let rows = shown("", &[]);
    let row = |key: &str| rows.iter().find(|row| row.key == key).unwrap();

    let grub = row("grub");

    assert_eq!(
        (grub.action, grub.admin, grub.link, grub.path.as_str()),
        ("edit", true, false, "/etc/default/grub")
    );
    assert_eq!(grub.name, "GRUB default");
    assert_eq!(grub.description, "GRUB configuration variables");
    assert_eq!(grub.url, "https://wiki.archlinux.org/title/GRUB");
    assert!(!grub.read_only);

    let bash = row("bash");

    assert_eq!(
        (bash.action, bash.admin, bash.path.as_str()),
        ("edit", false, "~/.bashrc")
    );

    let linked = row("os-release");

    assert_eq!((linked.action, linked.admin, linked.link), ("", true, true));

    let section = row("boot");

    assert_eq!(
        (section.name.as_str(), section.icon.as_str()),
        ("Bootloader", "system-reboot")
    );

    let folder = row("modprobe");

    assert_eq!(
        (folder.action, folder.count, folder.expanded),
        ("", 2, false)
    );
}

fn read_only() -> (ConfigFileCatalog, Listing) {
    let catalog = ConfigFileCatalog::parse(&format!("{CATALOG}{READ_ONLY}"), "test.toml").unwrap();
    let mut listing = Listing::default();

    listing.insert("passwd", Found::File { link: None });
    listing.insert("pam", Found::Folder(vec!["su".to_owned()]));
    listing.insert("grub", Found::File { link: None });
    listing.insert("bash", Found::File { link: None });
    (catalog, listing)
}

/// The page shows these files but the helper never writes them, so no
/// row offers an edit that could only fail. Their text is shown instead.
#[test]
fn a_read_only_file_and_the_files_of_a_read_only_folder_are_viewed() {
    let (catalog, listing) = read_only();

    let rows = rows(&catalog, &listing, &BTreeSet::from(["pam".to_owned()]), "");
    let shown: Vec<_> = rows
        .iter()
        .filter(|row| row.kind != RowKind::Section)
        .map(|row| (row.key.as_str(), row.action, row.admin, row.read_only))
        .collect();

    assert_eq!(
        shown,
        [
            ("grub", "edit", true, false),
            ("bash", "edit", false, false),
            ("passwd", "view", true, true),
            ("pam", "", true, true),
            ("pam/su", "view", true, true),
        ]
    );
}

#[test]
fn only_a_read_only_file_the_page_shows_is_viewed() {
    let (catalog, mut listing) = read_only();
    let view = |found: &Listing, key: &str| {
        view_target(&catalog, found, key).map(|path| path.as_str().to_owned())
    };

    assert_eq!(view(&listing, "passwd").as_deref(), Some("/etc/passwd"));
    assert_eq!(view(&listing, "pam/su").as_deref(), Some("/etc/pam.d/su"));

    for key in [
        "grub",
        "bash",
        "pam",
        "pam/sudo",
        "../etc/shadow",
        "shadow",
        "",
    ] {
        assert_eq!(view(&listing, key), None, "{key}");
    }

    // The helper refuses what the link leads to, so there is nothing to see.
    listing.insert(
        "passwd",
        Found::File {
            link: Some(Link::refused(
                "/nix/store/passwd",
                toolbox_plan::FetchRefusal::Outside,
            )),
        },
    );

    let rows = rows(&catalog, &listing, &BTreeSet::new(), "passwd");

    assert_eq!(view(&listing, "passwd"), None);
    assert_eq!(
        rows.iter().find(|row| row.key == "passwd").unwrap().action,
        ""
    );
}
