mod source;

use std::sync::Arc;

use egui::Theme;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use egui_kittest::{Harness, SnapshotResults};
use tokio::runtime::Runtime;
use toolbox_core::domains::{InstalledPackages, SyncPackages};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui::packages::{PackagesSetup, SampleAur};
use toolbox_gui::queue::Queue;
use toolbox_gui::{App, PreferencesSetup, Setup, snapshot};

use self::source::{Database, Log};
use crate::common::{Fixture, SMALLEST, TALL, WIDTH, packages, type_into};
use crate::system::{first_read, settle, step_until};

const INSTALLED: [&str; 5] = ["babl", "gegl", "gimp", "leftover", "paru"];
const OFFERED: [&str; 6] = ["babl", "gegl", "gimp", "inkscape", "krita", "leftover"];

fn system() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(packages(&INSTALLED)));
    snapshot.insert(SyncPackages(packages(&OFFERED)));
    snapshot
}

fn app(theme: Theme, language: Option<&str>) -> Fixture {
    app_with(theme, language, false)
}

/// The page on the fake database, with the dry run's sample AUR shown
/// when `aur`.
pub fn app_with(theme: Theme, language: Option<&str>, aur: bool) -> Fixture {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();
    let probed = system();
    let system_language = language.map_or(toolbox_gui::i18n::FALLBACK, |tag| tag.parse().unwrap());

    let mut harness = crate::common::window(TALL)
        .with_theme(theme)
        .build_eframe(move |cc| {
            let probe = move || (probed.clone(), Vec::new());
            App::new(
                cc,
                Setup {
                    queue: Queue::in_memory(ChangeSet::default()),
                    snapshot: Some(snapshot::spawn(
                        &handle,
                        toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
                        probe,
                    )),
                    packages: Some(PackagesSetup {
                        runtime: handle.clone(),
                        source: Arc::new(Database::captured()),
                        log: Arc::new(Log),
                        aur: Arc::new(SampleAur),
                    }),
                    system_language,
                    preferences: preferences(aur),
                    ..Setup::default()
                },
            )
        });

    first_read(&mut harness);
    let page = if language.is_some() {
        "Pakete"
    } else {
        "Packages"
    };

    harness.get_by_role_and_label(Role::Tab, page).click();

    step_until(&mut harness, "the index", |harness| listed(harness, "gimp"));
    settle(&mut harness);

    Fixture::new(harness, runtime)
}

fn preferences(aur: bool) -> PreferencesSetup {
    let mut preferences = PreferencesSetup::default();
    preferences.settings.packages.show_aur = aur;
    preferences
}

pub fn listed(harness: &Harness<'static, App>, package: &str) -> bool {
    harness
        .query_by_role_and_label(Role::ListItem, package)
        .is_some()
}

fn open_gimp(harness: &mut Harness<'static, App>) {
    harness
        .get_by_role_and_label(Role::ListItem, "gimp")
        .click();
    step_until(harness, "the details", |harness| {
        harness.query_by_label("https://www.gimp.org/").is_some()
    });
    settle(harness);
}

#[test]
fn the_list_starts_on_the_installed_packages() {
    let mut fixture = app(Theme::Dark, None);
    let harness = fixture.harness();

    assert!(listed(harness, "paru"));
    assert!(!listed(harness, "inkscape"));
    assert!(harness.query_by_label("Not from a repository").is_some());
}

#[test]
fn an_install_from_a_search_waits_in_the_queue() {
    let mut fixture = app(Theme::Dark, None);
    let harness = fixture.harness();

    harness.get_by_role_and_label(Role::Button, "All 7").click();
    harness.run();
    type_into(harness, "Search packages", "vector");
    assert!(listed(harness, "inkscape"));
    assert!(!listed(harness, "gimp"));

    harness.get_by_label("Install inkscape").click();
    harness.run();

    assert!(harness.query_by_label("Installs with Apply").is_some());
    assert!(harness.query_by_label("change queued").is_some());
}

#[test]
fn a_removal_names_what_goes_along_and_waits_in_the_queue() {
    let mut fixture = app(Theme::Dark, None);
    let harness = fixture.harness();
    open_gimp(harness);

    harness.get_by_label("Remove gimp").click();
    harness.run();
    assert!(
        harness
            .query_by_label("It takes along 2 dependencies that nothing else needs:")
            .is_some()
    );
    harness.snapshot("packages/remove_dark");

    harness
        .get_by_role_and_label(Role::Button, "Remove")
        .click();
    harness.run();
    assert!(harness.query_by_label("Take back gimp").is_some());
    assert!(harness.query_by_label("change queued").is_some());
}

#[test]
fn a_dependency_opens_its_own_page_and_back_returns_to_the_first() {
    let mut fixture = app(Theme::Dark, None);
    let harness = fixture.harness();
    open_gimp(harness);

    harness.get_by_role_and_label(Role::Link, "gegl").click();
    step_until(harness, "gegl's details", |harness| {
        harness.query_by_label("https://www.gegl.org/").is_some()
    });
    assert!(
        harness
            .query_by_role_and_label(Role::Link, "gimp")
            .is_some()
    );

    harness.get_by_label("Back to gimp").click();
    step_until(harness, "gimp again", |harness| {
        harness.query_by_label("https://www.gimp.org/").is_some()
    });

    harness.get_by_label("Back to Packages").click();
    harness.run();
    assert!(listed(harness, "gimp"));
}

#[test]
fn snapshots() {
    let mut results = SnapshotResults::new();

    let mut dark = app(Theme::Dark, None);
    let harness = dark.harness();
    results.add(harness.try_snapshot("packages/list_dark"));

    open_gimp(harness);
    results.add(harness.try_snapshot("packages/detail_dark"));

    harness.set_size(egui::vec2(SMALLEST.0, TALL));
    settle(harness);
    results.add(harness.try_snapshot("packages/detail_narrow_dark"));

    harness.set_size(egui::vec2(WIDTH, TALL));
    settle(harness);

    // The captured log holds one change of gimp, too few for Show all.
    let gimp = Database::captured().package("gimp");
    let button = format!("Files {}", gimp.installed.unwrap().files.len());
    harness
        .get_by_role_and_label(Role::Button, &button)
        .scroll_to_me();
    harness.run();
    harness.get_by_role_and_label(Role::Button, &button).click();
    harness.run();

    harness.get_by_label("Installed 3.2.6-2").scroll_to_me();
    settle(harness);
    results.add(harness.try_snapshot("packages/detail_unfolded_dark"));

    let mut light = app(Theme::Light, None);
    let bright = light.harness();
    results.add(bright.try_snapshot("packages/list_light"));
    open_gimp(bright);
    results.add(bright.try_snapshot("packages/detail_light"));

    let mut german = app(Theme::Dark, Some("de-DE"));
    let deutsch = german.harness();
    results.add(deutsch.try_snapshot("packages/list_german"));
    open_gimp(deutsch);
    results.add(deutsch.try_snapshot("packages/detail_german"));
}
