//! Entries that hand out root or the accounts are shown but never
//! written, so the helper's write table must not name them.

use super::super::{ConfigEntry, ConfigFileCatalog};
use super::{SMALL, SOURCE, changed, problem, shipped};
use crate::Problem;

fn with_read_only(path: &str, directory: bool) -> String {
    let folder = if directory { "\ndirectory = true" } else { "" };

    format!(
        "{SMALL}\n[[section]]\nid = \"accounts\"\ntitle = \"Accounts\"\nicon = \"system-users\"\n\n\
         [[section.entry]]\nid = \"locked\"\nname = \"Locked\"\npath = \"{path}\"\nread_only = true{folder}\n"
    )
}

#[test]
fn a_read_only_entry_outside_the_write_table_is_shown() {
    let catalog = ConfigFileCatalog::parse(&with_read_only("/etc/passwd", false), SOURCE).unwrap();
    let folder = ConfigFileCatalog::parse(&with_read_only("/etc/pam.d", true), SOURCE).unwrap();

    assert!(catalog.entry("locked").unwrap().is_read_only());
    assert!(folder.entry("locked").unwrap().is_read_only());
    assert!(!catalog.entry("grub-default").unwrap().is_read_only());
}

/// A line of the table would let the helper write what the page only
/// shows, and so hand out root.
#[test]
fn a_read_only_entry_in_the_write_table_is_refused() {
    for (path, directory) in [
        ("/etc/fstab", false),
        ("/etc/modprobe.d/nvidia.conf", false),
        ("/etc/modprobe.d/own.conf", false),
        ("/etc/modprobe.d", true),
    ] {
        let (id, found) = problem(&with_read_only(path, directory));

        assert_eq!(id, "locked");
        assert!(
            matches!(&found, Problem::ReadOnlyWritable(text) if text == path),
            "{path}: {found}"
        );
    }
}

/// Files of the user open in their own app, as the user, so nothing is
/// there to refuse.
#[test]
fn a_file_of_the_user_is_never_read_only() {
    let (id, found) = problem(&changed(
        "path = \"~/.bashrc\"",
        "path = \"~/.bashrc\"\nread_only = true",
    ));

    assert_eq!(id, "bash");
    assert!(matches!(found, Problem::ReadOnlyHome), "{found}");
}

#[test]
fn the_shipped_catalog_keeps_root_and_the_accounts_read_only() {
    let catalog = shipped();
    let read_only: Vec<&str> = catalog
        .entries()
        .filter(|entry| entry.is_read_only())
        .map(ConfigEntry::id)
        .collect();

    assert_eq!(
        read_only,
        [
            "pacman-hooks",
            "systemd-services",
            "udev-rules",
            "group",
            "passwd",
            "sudoers",
            "pam",
            "cron",
            "cron-daily",
            "cron-hourly",
            "cron-monthly",
            "cron-weekly",
        ]
    );
}
