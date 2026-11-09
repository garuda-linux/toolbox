use std::fs;

use toolbox_catalog::ConfigFileCatalog;

use super::{Readable, TARGET, readable, render};
use crate::catalogs::SOURCE_DIR;
use crate::root;

const CATALOG: &str = r#"
[[section]]
id = "system"
title = "System"
icon = "system-reboot"

[[section.entry]]
id = "sudoers"
name = "Sudoers"
path = "/etc/sudoers"
read_only = true

[[section.entry]]
id = "grub-default"
name = "GRUB default"
path = "/etc/default/grub"

[[section.entry]]
id = "pam"
name = "PAM"
path = "/etc/pam.d"
read_only = true
directory = true

[[section.entry]]
id = "group"
name = "Group"
path = "/etc/group"
read_only = true

[[section.entry]]
id = "bash"
name = "Bash"
path = "~/.bashrc"
"#;

fn table(files: &[&str], folders: &[&str]) -> Readable {
    Readable {
        files: files.iter().map(|&file| file.to_owned()).collect(),
        folders: folders.iter().map(|&folder| folder.to_owned()).collect(),
    }
}

/// The write table and the user's own files have rules of their own, so
/// only the read-only root-owned entries land here, sorted for the
/// helper's binary search.
#[test]
fn only_read_only_root_entries_are_listed_sorted() {
    let catalog = ConfigFileCatalog::parse(CATALOG, "config-files.toml").unwrap();

    assert_eq!(
        readable(&catalog),
        table(&["/etc/group", "/etc/sudoers"], &["/etc/pam.d"])
    );
}

/// `nix fmt` runs rustfmt over the file, so the task must write what
/// rustfmt leaves alone.
#[test]
fn a_short_table_stays_on_one_line_and_a_long_one_takes_a_line_per_file() {
    let text = render(&table(
        &["/etc/group"],
        &[
            "/etc/cron.d",
            "/etc/cron.daily",
            "/etc/pam.d",
            "/etc/systemd/system",
        ],
    ));

    assert!(
        text.contains("pub const READABLE_CONFIG_FILES: [&str; 1] = [\"/etc/group\"];\n"),
        "{text}"
    );
    assert!(
        text.contains(
            "pub const READABLE_CONFIG_FOLDERS: [&str; 4] = [\n    \"/etc/cron.d\",\n    \
             \"/etc/cron.daily\",\n    \"/etc/pam.d\",\n    \"/etc/systemd/system\",\n];\n"
        ),
        "{text}"
    );
}

#[test]
fn an_empty_table_is_an_empty_array() {
    let text = render(&Readable::default());

    assert!(
        text.contains("pub const READABLE_CONFIG_FILES: [&str; 0] = [];\n"),
        "{text}"
    );
}

/// What `cargo xtask catalog-readable --check` checks, so a stale table
/// fails the tests too.
#[test]
fn the_checked_in_table_is_the_one_of_the_shipped_catalog() {
    let catalog = ConfigFileCatalog::load(&root().join(SOURCE_DIR)).unwrap();
    let current = fs::read_to_string(root().join(TARGET)).unwrap();

    assert_eq!(current, render(&readable(&catalog)));
}
