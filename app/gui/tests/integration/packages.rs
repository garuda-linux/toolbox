use std::sync::Arc;

use egui::Theme;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use egui_kittest::{Harness, SnapshotResults};
use tokio::runtime::Runtime;
use toolbox_core::domains::{InstalledPackages, SyncPackages};
use toolbox_core::features::packages::{
    IndexEntry, InstallReason, Installed, InstalledDetails, LocalNode, OptionalDependency, Origin,
    PackageDetails, PackageIndex,
};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui::packages::{PackagesSetup, SampleAur};
use toolbox_gui::queue::Queue;
use toolbox_gui::{App, PreferencesSetup, Setup, snapshot};
use toolbox_plan::PackageName;
use toolbox_system::{PackageIndexSource, ProbeError, RecentLogSource};

use crate::common::{Fixture, SMALLEST, TALL, WIDTH, packages, type_into};
use crate::system::{first_read, settle, step_until};

const INSTALLED: [&str; 5] = ["babl", "gegl", "gimp", "leftover", "paru"];
const OFFERED: [&str; 6] = ["babl", "gegl", "gimp", "inkscape", "krita", "leftover"];

/// gimp with gegl and babl, a leftover dependency, paru built from the
/// AUR, and inkscape and krita on offer.
struct Database;

/// Six changes of gimp in pacman's log, more than the history shows
/// at first.
struct Log;

impl PackageIndexSource for Database {
    fn index(&self) -> Result<PackageIndex, ProbeError> {
        let dependency = Some(InstallReason::Dependency);
        let explicit = Some(InstallReason::Explicit);
        Ok(PackageIndex {
            entries: vec![
                entry(
                    "babl",
                    "Dynamic, any to any, pixel format conversion library",
                    dependency,
                ),
                entry("gegl", "Graph based image processing framework", dependency),
                entry("gimp", "GNU Image Manipulation Program", explicit),
                entry("inkscape", "Professional vector graphics editor", None),
                entry("krita", "Edit and paint images", None),
                entry("leftover", "A library nothing needs anymore", dependency),
                entry("paru", "Feature packed AUR helper", explicit),
            ],
            local: vec![
                node("babl", InstallReason::Dependency, &[]),
                node("gegl", InstallReason::Dependency, &["babl"]),
                node("gimp", InstallReason::Explicit, &["babl", "gegl>=0.4.50"]),
                node("leftover", InstallReason::Dependency, &[]),
                node("paru", InstallReason::Explicit, &[]),
            ],
        })
    }

    fn details(&self, package: &PackageName) -> Result<Option<PackageDetails>, ProbeError> {
        match package.as_str() {
            "gimp" => Ok(Some(gimp())),
            "gegl" => Ok(Some(gegl())),
            _ => Ok(None),
        }
    }
}

impl RecentLogSource for Log {
    fn recent(&self) -> Result<String, ProbeError> {
        Ok(
            "[2024-09-02T09:00:00+0200] [ALPM] installed gimp (2.10.36-1)\n\
            [2024-11-12T19:40:00+0100] [ALPM] upgraded gimp (2.10.36-1 -> 2.10.38-1)\n\
            [2025-01-20T07:05:00+0100] [ALPM] removed gimp (2.10.38-1)\n\
            [2025-03-17T10:00:00+0100] [ALPM] installed gimp (3.0.0-1)\n\
            [2025-04-02T18:30:00+0200] [ALPM] upgraded gimp (3.0.0-1 -> 3.0.2-1)\n\
            [2025-06-01T08:15:00+0200] [ALPM] upgraded gimp (3.0.2-1 -> 3.0.4-1)\n"
                .to_owned(),
        )
    }
}

fn gegl() -> PackageDetails {
    PackageDetails {
        name: name("gegl"),
        description: "Graph based image processing framework".to_owned(),
        version: "0.4.62-1".to_owned(),
        origin: Origin::Repository("extra".to_owned()),
        size: 9 * 1024 * 1024,
        url: Some("https://gegl.org/".to_owned()),
        licenses: vec!["GPL-3.0-or-later".to_owned()],
        groups: Vec::new(),
        depends: vec!["babl".to_owned()],
        optional: Vec::new(),
        required_by: vec!["gimp".to_owned()],
        provides: Vec::new(),
        conflicts: Vec::new(),
        replaces: Vec::new(),
        installed: Some(InstalledDetails {
            version: "0.4.62-1".to_owned(),
            reason: InstallReason::Dependency,
            date: Some(1_750_000_000),
            files: vec!["/usr/lib/libgegl-0.4.so".to_owned()],
        }),
    }
}

fn gimp() -> PackageDetails {
    PackageDetails {
        name: name("gimp"),
        description: "GNU Image Manipulation Program".to_owned(),
        version: "3.0.4-1".to_owned(),
        origin: Origin::Repository("extra".to_owned()),
        size: 37 * 1024 * 1024,
        url: Some("https://www.gimp.org/".to_owned()),
        licenses: vec![
            "GPL-3.0-or-later".to_owned(),
            "LGPL-3.0-or-later".to_owned(),
        ],
        groups: Vec::new(),
        depends: vec!["babl".to_owned(), "gegl>=0.4.50".to_owned()],
        optional: vec![OptionalDependency {
            name: "ghostscript".to_owned(),
            reason: "for postscript support".to_owned(),
        }],
        required_by: Vec::new(),
        provides: Vec::new(),
        conflicts: vec!["gimp-plugin-wavelet-decompose".to_owned()],
        replaces: Vec::new(),
        installed: Some(InstalledDetails {
            version: "3.0.4-1".to_owned(),
            reason: InstallReason::Explicit,
            date: Some(1_750_000_000),
            files: vec![
                "/usr/".to_owned(),
                "/usr/bin/".to_owned(),
                "/usr/bin/gimp".to_owned(),
                "/usr/share/applications/gimp.desktop".to_owned(),
            ],
        }),
    }
}

fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap()
}

fn entry(text: &str, description: &str, installed: Option<InstallReason>) -> IndexEntry {
    IndexEntry {
        name: name(text),
        description: description.to_owned(),
        version: "3.0.4-1".to_owned(),
        origin: if text == "paru" {
            Origin::Foreign
        } else {
            Origin::Repository("extra".to_owned())
        },
        size: 37 * 1024 * 1024,
        installed: installed.map(|reason| Installed {
            version: "3.0.4-1".to_owned(),
            reason,
        }),
    }
}

fn node(text: &str, reason: InstallReason, depends: &[&str]) -> LocalNode {
    LocalNode {
        name: name(text),
        depends: depends.iter().map(|&dep| dep.to_owned()).collect(),
        provides: Vec::new(),
        reason,
    }
}

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
                        source: Arc::new(Database),
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

    harness
        .get_by_role_and_label(Role::Link, "gegl>=0.4.50")
        .click();
    step_until(harness, "gegl's details", |harness| {
        harness.query_by_label("https://gegl.org/").is_some()
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
    for button in ["Show all", "Files 4"] {
        let shown = harness.get_by_role_and_label(Role::Button, button);
        shown.scroll_to_me();
        harness.run();
        harness.get_by_role_and_label(Role::Button, button).click();
        harness.run();
    }

    harness.get_by_label("Installed 2.10.36-1").scroll_to_me();
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
