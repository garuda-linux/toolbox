use std::fs;
use std::path::{Path, PathBuf};

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use tempfile::TempDir;
use tokio::runtime::Runtime;
use toolbox_core::domains::{InstalledPackages, SyncPackages};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui::catalogs::Catalogs;
use toolbox_gui::pictures::{PictureFinder, Pictures};
use toolbox_gui::queue::Queue;
use toolbox_gui::{App, Setup, snapshot};
use toolbox_plan::PackageName;

use crate::common::{HEIGHT, packages, palette, type_into, window};
use crate::system::first_read;

pub struct Table(pub Vec<(&'static str, PathBuf)>);

impl PictureFinder for Table {
    fn find(&mut self, _theme: &str, package: &PackageName) -> Option<PathBuf> {
        self.0
            .iter()
            .find(|(name, _)| *name == package.as_str())
            .map(|(_, file)| file.clone())
    }
}

pub fn source_tree() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog")
}

/// A system that offers a handful of apps and has Kate installed, so the
/// lists stay shorter than the window.
pub fn system() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(packages(&["kate"])));
    snapshot.insert(SyncPackages(packages(&[
        "aerc",
        "firefox",
        "gimp",
        "kate",
        "okular",
        "stellarium",
    ])));
    snapshot
}

pub fn discover_with(catalogs: Catalogs, system: Snapshot) -> (Harness<'static, App>, Runtime) {
    pictured(catalogs, system, None)
}

pub fn pictured(
    catalogs: Catalogs,
    system: Snapshot,
    pictures: Option<Table>,
) -> (Harness<'static, App>, Runtime) {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();

    let mut harness = window(HEIGHT).build_eframe(move |cc| {
        let probe = move || (system.clone(), Vec::new());
        let finder = pictures.map(|table| {
            let found = Pictures::spawn(cc.egui_ctx.clone(), table);
            found.set_theme("test");
            found
        });

        let setup = Setup {
            queue: Queue::in_memory(ChangeSet::default()),
            snapshot: Some(snapshot::spawn(
                &handle,
                toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
                probe,
            )),
            pictures: finder,
            catalogs,
            ..Setup::default()
        };

        App::new(cc, setup)
    });

    first_read(&mut harness);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Num5);
    harness.run();
    (harness, runtime)
}

pub fn source_catalogs() -> Catalogs {
    Catalogs::load(&source_tree())
}

pub fn discover() -> (Harness<'static, App>, Runtime) {
    discover_with(source_catalogs(), system())
}

pub fn chip(harness: &mut Harness<'static, App>, title: &str) {
    harness
        .get_all_by_role(Role::Button)
        .find(|node| {
            node.accesskit_node()
                .label()
                .is_some_and(|label| label.starts_with(title))
        })
        .unwrap()
        .click();

    // Pictures decode on other threads, which repaints a few more frames.
    harness.run_steps(8);
}

fn switch_shown(harness: &Harness<'static, App>, title: &str) -> bool {
    harness
        .query_by_role_and_label(Role::CheckBox, title)
        .is_some()
}

#[test]
fn every_category_shows_and_a_chip_picks_one() {
    let (mut harness, _runtime) = discover();
    assert!(switch_shown(&harness, "Kate"));
    assert!(switch_shown(&harness, "Firefox"));

    chip(&mut harness, "Internet");

    assert!(switch_shown(&harness, "Firefox"));
    assert!(switch_shown(&harness, "aerc"));
    assert!(!switch_shown(&harness, "Kate"));
}

#[test]
fn a_switch_queues_an_app_and_takes_it_back() {
    let (mut harness, _runtime) = discover();
    chip(&mut harness, "Internet");

    harness
        .get_by_role_and_label(Role::CheckBox, "Firefox")
        .click();
    harness.run();
    assert!(harness.query_by_label("change queued").is_some());
    assert!(harness.query_by_label("Turns on with Apply").is_some());

    harness
        .get_by_role_and_label(Role::CheckBox, "Firefox")
        .click();
    harness.run();
    assert!(harness.query_by_label("change queued").is_none());
}

#[test]
fn a_search_looks_through_every_category() {
    let (mut harness, _runtime) = discover();

    let before = harness.query_all_by_label("Internet").count();

    type_into(&mut harness, "Search apps", "gecko");

    assert!(switch_shown(&harness, "Firefox"));
    assert!(!switch_shown(&harness, "Kate"));
    assert_eq!(
        harness.query_all_by_label("Internet").count(),
        before + 1,
        "the row names its category"
    );
}

#[test]
fn a_search_without_hits_says_so_and_clears() {
    let (mut harness, _runtime) = discover();

    type_into(&mut harness, "Search apps", "zzzz-nothing");
    assert!(
        harness
            .query_by_label("No app matches “zzzz-nothing”")
            .is_some()
    );

    harness
        .get_by_role_and_label(Role::Button, "Clear search")
        .click();
    harness.run();
    assert!(switch_shown(&harness, "Kate"));
}

#[test]
fn selected_first_puts_installed_apps_on_top() {
    let (mut harness, _runtime) = discover();
    type_into(&mut harness, "Search apps", "o");
    let first = |shown: &Harness<'static, App>| {
        shown
            .get_all_by_role(Role::CheckBox)
            .next()
            .and_then(|node| node.accesskit_node().label())
            .unwrap()
    };

    assert_ne!(first(&harness), "Kate");

    harness
        .get_by_role_and_label(Role::ComboBox, "Sort apps")
        .click();
    harness.run();
    harness
        .get_by_role_and_label(Role::ListBoxOption, "Selected first")
        .click();
    harness.run();

    assert_eq!(first(&harness), "Kate");
}

#[test]
fn the_row_menu_holds_the_website() {
    let (mut harness, _runtime) = discover();

    harness
        .get_by_role_and_label(Role::Button, "More for Kate")
        .click();
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::MenuItem, "Open website")
            .is_some()
    );
}

#[test]
fn a_broken_category_leaves_the_others_with_a_notice() {
    let dir = TempDir::new().unwrap();
    let apps = dir.path().join("apps");
    fs::create_dir_all(&apps).unwrap();
    fs::copy(
        source_tree().join("apps/science.toml"),
        apps.join("science.toml"),
    )
    .unwrap();

    let (harness, _runtime) = discover_with(Catalogs::load(dir.path()), system());

    assert!(
        harness
            .query_by_label("Some categories could not be loaded. The others are shown. A catalog file could not be read.")
            .is_some()
    );
    assert!(switch_shown(&harness, "Stellarium"));
}

#[test]
fn the_palette_finds_an_app_and_opens_it_here() {
    let (mut harness, _runtime) = discover();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Num1);
    harness.run();

    palette(&mut harness, "firef");
    harness.key_press(Key::Enter);
    harness.run();

    assert!(switch_shown(&harness, "Firefox"));
    assert!(!switch_shown(&harness, "Kate"));

    let focused: Vec<String> = harness
        .query_all_by_role(Role::ListItem)
        .filter(egui_kittest::Node::is_focused)
        .filter_map(|row| row.accesskit_node().label())
        .collect();

    assert_eq!(focused, ["Firefox"], "the picked app's row");
}

#[test]
fn during_a_search_the_chips_count_the_matches_and_still_pick() {
    let (mut harness, _runtime) = discover();
    type_into(&mut harness, "Search apps", "gecko");

    let internet = harness
        .get_all_by_role(Role::Button)
        .filter_map(|node| node.accesskit_node().label())
        .find(|label| label.starts_with("Internet"))
        .unwrap();

    assert_eq!(internet, "Internet 1");

    chip(&mut harness, "Documents");
    assert!(!switch_shown(&harness, "Firefox"));

    chip(&mut harness, "All");
    assert!(switch_shown(&harness, "Firefox"));
}

#[test]
fn without_the_installed_packages_one_banner_offers_to_read_again() {
    let (harness, _runtime) = discover_with(source_catalogs(), Snapshot::default());

    assert!(
        harness
            .query_by_label_contains("The installed apps could not be read")
            .is_some()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Read again")
            .is_some()
    );
}
