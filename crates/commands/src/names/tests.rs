use toolbox_catalog::ToggleCatalog;
use toolbox_core::Snapshot;
use toolbox_core::domains::{Accounts, AvailableShells, Group, User};
use toolbox_core::features::locales::{SupportedLocales, parse_supported};
use toolbox_plan::{GroupName, ShellPath, UserName};

use super::{
    app_categories, entry_ids, group_names, guide_pages, locale_names, pack_names, package_names,
    page_names, parameter_names, shell_names, switch_ids, task_ids, user_names,
};
use crate::locales::PackRow;

const CATALOG: &str = r#"
[[section]]
id = "containers"
title = "Containers"
icon = "x"

[[section.entry]]
id = "docker"
title = "Docker"
description = "The engine."
check = { package = "docker" }
"#;

const GRUB_CFG: &str = "\
menuentry 'Garuda Linux' $menuentry_id_option 'gnulinux-simple-1' {
}
submenu 'Advanced options' $menuentry_id_option 'gnulinux-advanced-1' {
	menuentry 'Garuda Linux, with Linux linux-lts' $menuentry_id_option 'gnulinux-lts-1' {
	}
}
";

fn user(name: &str, uid: u32) -> User {
    User {
        name: UserName::new(name).unwrap(),
        uid,
        gid: uid,
        gecos: String::new(),
        home: format!("/home/{name}"),
        shell: "/usr/bin/fish".to_owned(),
    }
}

#[test]
fn switches_and_tasks_come_by_their_ids() {
    let catalog = ToggleCatalog::parse(CATALOG, "test.toml").unwrap();

    assert_eq!(switch_ids(&[&catalog]), ["docker"]);
    assert!(task_ids().contains(&"clean-package-cache".to_owned()));
    assert!(page_names().contains(&"maintenance".to_owned()));
}

#[test]
fn boot_entries_leave_out_the_submenus() {
    let ids = entry_ids(GRUB_CFG);

    assert!(ids.contains(&"gnulinux-simple-1".to_owned()));
    assert!(!ids.iter().any(|id| id == "gnulinux-advanced-1"));
}

#[test]
fn accounts_name_the_people_and_every_group() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(Accounts {
        users: vec![user("ada", 1001), user("daemon", 2)],
        groups: vec![Group {
            name: GroupName::new("wheel").unwrap(),
            gid: 998,
            members: [UserName::new("ada").unwrap()].into(),
        }],
    });

    assert_eq!(user_names(&snapshot), ["ada"]);
    assert_eq!(group_names(&snapshot), ["wheel"]);
}

#[test]
fn locales_come_as_the_commands_take_them() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(SupportedLocales(parse_supported("de_DE.UTF-8 UTF-8\n")));

    assert_eq!(locale_names(&snapshot), ["de_DE.UTF-8"]);
}

#[test]
fn categories_are_the_catalog_file_names() {
    let categories = app_categories();

    assert_eq!(categories.len(), toolbox_catalog::APPS.len());
    assert!(categories.contains(&"internet".to_owned()));
    assert!(categories.contains(&"games".to_owned()));
}

#[test]
fn shells_come_as_full_paths() {
    let shells = AvailableShells(vec![
        ShellPath::new("/bin/sh").unwrap(),
        ShellPath::new("/usr/bin/fish").unwrap(),
    ]);

    assert_eq!(shell_names(&shells), ["/bin/sh", "/usr/bin/fish"]);
}

#[test]
fn nothing_read_gives_no_names() {
    let none: Vec<String> = Vec::new();

    assert_eq!(user_names(&Snapshot::default()), none);
    assert_eq!(locale_names(&Snapshot::default()), none);
    assert_eq!(entry_ids(""), none);
}

#[test]
fn packages_come_by_their_names() {
    let found = ["gimp", "git"]
        .map(|name| toolbox_plan::PackageName::new(name).unwrap())
        .into();

    assert_eq!(package_names(&found), ["gimp", "git"]);
}

#[test]
fn packs_split_into_installed_and_not() {
    let pack = |package: &str, installed| PackRow {
        package: package.to_owned(),
        program: "firefox".to_owned(),
        locale: "de".to_owned(),
        installed,
    };

    let rows = [
        pack("firefox-i18n-de", false),
        pack("thunderbird-i18n-de", true),
    ];

    assert_eq!(pack_names(&rows, false), ["firefox-i18n-de"]);
    assert_eq!(pack_names(&rows, true), ["thunderbird-i18n-de"]);
}

#[test]
fn guide_pages_come_as_guide_show_takes_them() {
    let pages = guide_pages();

    assert!(pages.contains(&"cli".to_owned()), "{pages:?}");
    assert!(pages.iter().all(|page| !page.contains(".md")));
}

#[test]
fn no_boot_menu_gives_no_parameters() {
    assert_eq!(parameter_names(&Snapshot::default()), Vec::<String>::new());
}

#[test]
fn the_scheme_names_are_system_then_the_modes_schemes() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog");
    let catalog = toolbox_catalog::ColourSchemeCatalog::load(&dir).unwrap();

    let dark = super::scheme_names(&catalog, true);
    let light = super::scheme_names(&catalog, false);

    assert_eq!(dark.first().map(String::as_str), Some("system"));
    assert!(dark.contains(&"nord".to_owned()));
    assert!(!dark.contains(&"nord-light".to_owned()));
    assert_eq!(light.get(1).map(String::as_str), Some("catppuccin-latte"));
}
