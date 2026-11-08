mod dialogs;
mod status;
mod wiki;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_catalog::ConfigFileCatalog;
use toolbox_core::features::configs::session::{Changes, Compared, EditedFile, Fetched, Session};
use toolbox_core::features::configs::viewing::{Viewed, Viewing};
use toolbox_core::features::configs::{Found, Link, Listing};
use toolbox_plan::{ConfigFilePath, ConfigText, EditCopy, Editor, FetchRefusal};

use super::ConfigsPage;
use crate::command::{Command, ConfigsCommand};
use crate::configs::Configs;
use crate::pages::Refusals;
use crate::testing::{PageFixture, PageState};

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
id = "os-release"
name = "OS release"
path = "/etc/os-release"

[[section]]
id = "system"
title = "System"
icon = "preferences-system"

[[section.entry]]
id = "modprobe"
name = "modprobe.d"
path = "/etc/modprobe.d"
directory = true

[[section.entry]]
id = "sudoers"
name = "sudoers"
path = "/etc/sudoers"
read_only = true

[[section.entry]]
id = "bash"
name = "Bash"
path = "~/.bashrc"
"#;

type Shown = Harness<'static, PageState<ConfigsPage>>;

fn path(text: &str) -> ConfigFilePath {
    ConfigFilePath::new(text).unwrap()
}

fn text(content: &str) -> ConfigText {
    ConfigText::new(content).unwrap()
}

fn catalog() -> ConfigFileCatalog {
    ConfigFileCatalog::parse(CATALOG, "test.toml").unwrap()
}

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
        Found::Folder(vec!["audio.conf".to_owned(), "nvidia.conf".to_owned()]),
    );
    listing.insert("sudoers", Found::File { link: None });
    listing.insert("bash", Found::File { link: None });
    listing
}

pub fn reviewing() -> Session {
    reviewing_of(path("/etc/default/grub").into())
}

pub fn reviewing_of(file: EditedFile) -> Session {
    let mut session = Session::default();
    session.start(file.clone()).unwrap();
    session.fetched(Ok(Fetched {
        editor: Editor::Micro,
        copy: EditCopy::new("/run/user/1000/garuda-toolbox/edit/grub").unwrap(),
        original: text("GRUB_TIMEOUT=5\n"),
        target: file,
    }));
    session.plan_started();
    session.plan_finished(true);
    session.compared(Ok(Compared::Changed(Changes {
        original: text("GRUB_TIMEOUT=5\n"),
        edited: text("GRUB_TIMEOUT=2\n"),
        plain: "@@ -1,1 +1,1 @@\n-GRUB_TIMEOUT=5\n+GRUB_TIMEOUT=2\n".to_owned(),
        colored: String::new(),
        added: 1,
        removed: 1,
    })));
    session
}

pub fn viewing() -> Viewing {
    let mut viewing = Viewing::default();
    let number = viewing.start(path("/etc/sudoers")).unwrap();
    viewing.read(
        number,
        Ok(Viewed {
            path: path("/etc/sudoers"),
            content: text("root ALL=(ALL:ALL) ALL\n"),
        }),
    );
    viewing
}

pub fn page(configs: Configs, refusals: Refusals) -> Shown {
    let mut harness = PageFixture {
        refusals,
        configs,
        ..PageFixture::default()
    }
    .harness(ConfigsPage::default());

    harness.run();
    harness
}

pub fn listed() -> Configs {
    Configs::showing(catalog(), listing())
}

pub fn sent(harness: &Shown) -> Vec<Command> {
    harness.state().fixture.sent()
}

#[test]
fn the_files_show_by_section_with_their_status() {
    let harness = page(listed(), Refusals::default());

    for label in [
        "Bootloader",
        "System",
        "GRUB default",
        "/etc/default/grub",
        "2 files",
    ] {
        assert!(shows(&harness, label), "{label}");
    }

    let line = |text| shows(&harness, text);
    assert!(line("Saving asks for an administrator password"));
    assert!(line(
        "Only shown here: it decides who may administer the system or what runs as root"
    ));
    assert!(line(
        "Not edited here: the link leads out of the system's config folders, to a file of a package or a generated one"
    ));
}

#[test]
fn system_and_home_files_are_edited_a_guarded_one_viewed_and_a_home_one_opened_in_its_app() {
    let mut harness = page(listed(), Refusals::default());

    for edit in harness.get_all_by_label("Edit") {
        edit.click();
    }

    harness.run();
    harness.get_by_label("View").click();
    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "More actions for Bash")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(Role::MenuItem, "Open in app")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [
            Command::from(ConfigsCommand::Edit("grub".to_owned())),
            Command::from(ConfigsCommand::Edit("bash".to_owned())),
            Command::from(ConfigsCommand::View("sudoers".to_owned())),
            Command::from(ConfigsCommand::Open("bash".to_owned())),
        ]
    );
}

#[test]
fn a_folders_files_start_where_the_folder_starts() {
    let mut harness = page(listed(), Refusals::default());
    harness.get_by_label("Show the files").click();
    harness.run();

    let left = |text: &str| {
        toolbox_gui_kit::testing::painted_text(&harness, text)
            .unwrap_or_else(|| panic!("{text} is painted"))
            .pos
            .x
    };

    let (folder, file) = (left("modprobe.d"), left("audio.conf"));

    assert!(
        (folder - file).abs() < 0.5,
        "the folder starts at {folder}, its file at {file}"
    );
}

#[test]
fn a_folder_shows_its_files_once_opened() {
    let mut harness = page(listed(), Refusals::default());

    assert!(!shows(&harness, "audio.conf"));

    harness.get_by_label("Show the files").click();
    harness.run();

    assert!(shows(&harness, "audio.conf"));

    assert_eq!(harness.query_all_by_label("Edit").count(), 4);
}

#[test]
fn a_search_without_a_match_says_so() {
    let mut harness = page(listed(), Refusals::default());

    harness.get_by_label("Search config files").click();
    harness.run();
    harness
        .get_by_label("Search config files")
        .type_text("klingon");
    harness.run();

    assert!(shows(&harness, "No config file matches"));
    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Clear search")
            .is_some()
    );
}

#[test]
fn before_the_first_look_the_page_says_it_looks() {
    let harness = page(Configs::default(), Refusals::default());

    assert!(shows(&harness, "Looking for config files…"));
}

/// Also true when a dialog's or a row's name repeats the label.
pub fn shows(harness: &Shown, label: &str) -> bool {
    harness.query_all_by_label(label).next().is_some()
}

#[test]
fn a_section_without_a_known_icon_shows_a_plain_text_file() {
    assert_eq!(
        super::list::section_icon("no-such-icon").name(),
        "text-x-generic"
    );
    assert_eq!(
        super::list::section_icon("drive-harddisk-root").name(),
        "drive-harddisk-root"
    );
}

#[test]
fn a_file_with_nothing_to_say_has_no_empty_line_under_its_path() {
    let harness = page(listed(), Refusals::default());
    let height = |name| {
        harness
            .get_by_role_and_label(Role::ListItem, name)
            .rect()
            .height()
    };

    assert!(
        height("Bash") + 10.0 < height("GRUB default"),
        "Bash is {} tall, GRUB default with its status line {}",
        height("Bash"),
        height("GRUB default")
    );
}
