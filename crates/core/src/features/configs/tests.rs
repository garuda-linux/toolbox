mod actions;
mod links;

use std::collections::BTreeSet;

use toolbox_catalog::ConfigFileCatalog;
use toolbox_plan::{
    ConfigFilePath, ConfigText, ConfigWrite, EditCopy, Editor, FetchRefusal, HomePath, HomeWrite,
    PlannedAction, RootAction, UserAction,
};

use super::session::EditWrite;
use super::{
    Found, Link, Listing, Row, Target, choose_editor, edit_plan, rows, target, write_plan,
};

const CATALOG: &str = r#"
[[section]]
id = "boot"
title = "Bootloader"
icon = "system-reboot"

[[section.entry]]
id = "grub"
name = "GRUB default"
path = "/etc/default/grub"
description = "GRUB configuration variables"
url = "https://wiki.archlinux.org/title/GRUB"

[[section.entry]]
id = "grub-cfg"
name = "GRUB config"
path = "/boot/grub/grub.cfg"

[[section]]
id = "system"
title = "System"
icon = "preferences-system"

[[section.entry]]
id = "os-release"
name = "OS release"
path = "/etc/os-release"

[[section.entry]]
id = "modprobe"
name = "modprobe.d"
path = "/etc/modprobe.d"
description = "Kernel module options"
url = "https://wiki.archlinux.org/title/Kernel_module"
directory = true

[[section]]
id = "user"
title = "User environment"
icon = "utilities-terminal"

[[section.entry]]
id = "bash"
name = "Bash"
path = "~/.bashrc"

[[section.entry]]
id = "units"
name = "User systemd units"
path = "~/.config/systemd/user"
directory = true
"#;

fn catalog() -> ConfigFileCatalog {
    ConfigFileCatalog::parse(CATALOG, "test.toml").unwrap()
}

/// GRUB's file, a linked `os-release`, two module files, the shell's file
/// and one user unit. `grub.cfg` is missing.
fn listing() -> Listing {
    let mut listing = Listing::default();

    listing.insert("grub", Found::File { link: None });
    listing.insert(
        "os-release",
        Found::File {
            link: Some(Link::refused("/usr/lib/os-release", FetchRefusal::Outside)),
        },
    );
    listing.insert(
        "modprobe",
        Found::Folder(vec!["amdgpu.conf".to_owned(), "nvidia.conf".to_owned()]),
    );
    listing.insert("bash", Found::File { link: None });
    listing.insert("units", Found::Folder(vec!["sync.service".to_owned()]));
    listing
}

fn shown(search: &str, expanded: &[&str]) -> Vec<Row> {
    let expanded: BTreeSet<String> = expanded.iter().map(|id| (*id).to_owned()).collect();

    rows(&catalog(), &listing(), &expanded, search)
}

fn keys(rows: &[Row]) -> Vec<(&str, &str)> {
    rows.iter()
        .map(|row| (row.kind.into(), row.key.as_str()))
        .collect()
}

#[test]
fn only_existing_files_show_under_their_sections() {
    let rows = shown("", &[]);

    assert_eq!(
        keys(&rows),
        [
            ("section", "boot"),
            ("file", "grub"),
            ("section", "system"),
            ("file", "os-release"),
            ("folder", "modprobe"),
            ("section", "user"),
            ("file", "bash"),
            ("folder", "units"),
        ]
    );
}

#[test]
fn a_section_without_existing_files_is_hidden() {
    let mut listing = Listing::default();

    listing.insert("bash", Found::File { link: None });

    let rows = rows(&catalog(), &listing, &BTreeSet::new(), "");

    assert_eq!(keys(&rows), [("section", "user"), ("file", "bash")]);
    assert_eq!(
        super::rows(&catalog(), &Listing::default(), &BTreeSet::new(), ""),
        []
    );
}

#[test]
fn a_folder_without_files_is_hidden() {
    let mut listing = Listing::default();

    listing.insert("modprobe", Found::Folder(Vec::new()));
    listing.insert("bash", Found::File { link: None });

    for search in ["", "modprobe"] {
        assert!(
            !rows(&catalog(), &listing, &BTreeSet::new(), search)
                .iter()
                .any(|row| row.key == "modprobe"),
            "{search}"
        );
    }
}

#[test]
fn an_expanded_folder_lists_its_files() {
    let rows = shown("", &["modprobe", "units"]);

    assert_eq!(
        keys(&rows),
        [
            ("section", "boot"),
            ("file", "grub"),
            ("section", "system"),
            ("file", "os-release"),
            ("folder", "modprobe"),
            ("child", "modprobe/amdgpu.conf"),
            ("child", "modprobe/nvidia.conf"),
            ("section", "user"),
            ("file", "bash"),
            ("folder", "units"),
            ("child", "units/sync.service"),
        ]
    );

    let child = rows
        .iter()
        .find(|row| row.key == "modprobe/nvidia.conf")
        .unwrap();

    assert_eq!(
        (
            child.name.as_str(),
            child.path.as_str(),
            child.action,
            child.admin
        ),
        ("nvidia.conf", "/etc/modprobe.d/nvidia.conf", "edit", true)
    );

    // Each file of a folder repeats the folder's description and link.
    assert_eq!(
        (child.description.as_str(), child.url.as_str()),
        (
            "Kernel module options",
            "https://wiki.archlinux.org/title/Kernel_module"
        )
    );

    let unit = rows
        .iter()
        .find(|row| row.key == "units/sync.service")
        .unwrap();

    assert_eq!(
        (unit.path.as_str(), unit.action, unit.admin),
        ("~/.config/systemd/user/sync.service", "edit", false)
    );
}

#[test]
fn the_search_looks_at_names_paths_descriptions_and_folder_files() {
    assert_eq!(
        keys(&shown("grub", &[])),
        [("section", "boot"), ("file", "grub")]
    );
    assert_eq!(
        keys(&shown("  BASHRC ", &[])),
        [("section", "user"), ("file", "bash")]
    );
    assert_eq!(
        keys(&shown("variables", &[])),
        [("section", "boot"), ("file", "grub")]
    );
    assert_eq!(
        keys(&shown("nvidia", &[])),
        [
            ("section", "system"),
            ("folder", "modprobe"),
            ("child", "modprobe/nvidia.conf"),
        ]
    );
    assert_eq!(
        keys(&shown("module options", &[])),
        [
            ("section", "system"),
            ("folder", "modprobe"),
            ("child", "modprobe/amdgpu.conf"),
            ("child", "modprobe/nvidia.conf"),
        ]
    );
    assert_eq!(shown("no such file", &[]), []);
}

#[test]
fn a_key_resolves_only_to_what_the_page_shows() {
    let catalog = catalog();
    let listing = listing();
    let resolve = |key: &str| target(&catalog, &listing, key);

    assert_eq!(
        resolve("grub"),
        Some(Target::Root(
            ConfigFilePath::new("/etc/default/grub").unwrap()
        ))
    );
    assert_eq!(resolve("bash").unwrap().shown(), "~/.bashrc");
    assert_eq!(
        resolve("modprobe/nvidia.conf").unwrap().shown(),
        "/etc/modprobe.d/nvidia.conf"
    );
    assert_eq!(
        resolve("units/sync.service").unwrap().shown(),
        "~/.config/systemd/user/sync.service"
    );
    assert_eq!(
        resolve("units/sync.service").unwrap().file_name(),
        "sync.service"
    );
    assert_eq!(resolve("grub").unwrap().file_name(), "grub");

    for key in [
        "",
        "nothing",
        "grub-cfg",
        "modprobe",
        "modprobe/",
        "modprobe/other.conf",
        "modprobe/../shadow",
        "modprobe/nvidia.conf/x",
        "grub/grub",
        "bash/.bashrc",
        "/etc/shadow",
        "units/../../.ssh/id_ed25519",
    ] {
        assert_eq!(resolve(key), None, "{key}");
    }
}

#[test]
fn the_editor_comes_from_the_variables_only_as_a_pick_from_the_list() {
    let all = |_| true;
    let only = |wanted: &'static [Editor]| move |editor| wanted.contains(&editor);

    assert_eq!(
        choose_editor(Some("nvim"), Some("nano"), all),
        Some(Editor::Neovim)
    );
    assert_eq!(
        choose_editor(Some("code --wait"), Some("vim"), all),
        Some(Editor::Vim)
    );
    assert_eq!(
        choose_editor(Some("kate"), Some("sh -c evil"), all),
        Some(Editor::Micro)
    );
    assert_eq!(choose_editor(None, None, all), Some(Editor::Micro));
    assert_eq!(
        choose_editor(None, None, only(&[Editor::Nano, Editor::Vi])),
        Some(Editor::Nano)
    );
    assert_eq!(
        choose_editor(None, None, only(&[Editor::Vi])),
        Some(Editor::Vi)
    );
    assert_eq!(
        choose_editor(Some("helix"), Some("nvim"), only(&[Editor::Neovim])),
        Some(Editor::Neovim)
    );
    assert_eq!(
        choose_editor(Some("emacs"), None, only(&[Editor::Vim])),
        None
    );
    assert_eq!(choose_editor(Some("vim"), Some("vim"), |_| false), None);
}

#[test]
fn the_plans_are_one_user_step_and_one_root_step() {
    let copy = EditCopy::new("/run/user/1000/garuda-toolbox/edit/grub").unwrap();
    let edit = edit_plan(Editor::Micro, copy.clone());

    assert_eq!(
        edit.steps()
            .iter()
            .map(|step| &step.action)
            .collect::<Vec<_>>(),
        [&PlannedAction::User(UserAction::EditFile {
            editor: Editor::Micro,
            copy
        })]
    );

    let text = ConfigText::new("a\n").unwrap();
    let write = ConfigWrite {
        path: ConfigFilePath::new("/etc/default/grub").unwrap(),
        original: text.fingerprint(),
        content: text,
    };

    assert_eq!(
        write_plan(EditWrite::Root(write.clone()))
            .steps()
            .iter()
            .map(|step| &step.action)
            .collect::<Vec<_>>(),
        [&PlannedAction::Root(RootAction::WriteConfigFile { write })]
    );
}

#[test]
fn a_file_of_the_user_is_written_back_as_the_user() {
    let text = ConfigText::new("a\n").unwrap();
    let write = HomeWrite {
        path: HomePath::new(".bashrc").unwrap(),
        original: text.fingerprint(),
        content: text,
    };

    assert_eq!(
        write_plan(EditWrite::Home(write.clone()))
            .steps()
            .iter()
            .map(|step| &step.action)
            .collect::<Vec<_>>(),
        [&PlannedAction::User(UserAction::WriteHomeFile { write })]
    );
}
