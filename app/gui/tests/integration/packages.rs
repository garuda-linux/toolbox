mod source;

use std::sync::Arc;

use egui::Theme;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use egui_kittest::{Harness, SnapshotResults};
use tokio::runtime::Runtime;
use toolbox_aur::AurSource;
use toolbox_core::domains::{InstalledPackages, SyncPackages};
use toolbox_core::features::packages::PackageGraph;
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui::packages::{PackagesSetup, SampleAur};
use toolbox_gui::queue::Queue;
use toolbox_gui::{App, PreferencesSetup, Setup, snapshot};
use toolbox_state::aur::{AurRoute, RouteKind};

use self::source::{Database, Log};
use crate::common::{Fixture, SMALLEST, TALL, WIDTH, packages, queued_shown, type_into};
use crate::system::{first_read, settle, step_until};

const INSTALLED: [&str; 5] = ["babl", "gegl", "gimp", "leftover", "paru"];
const OFFERED: [&str; 6] = ["babl", "gegl", "gimp", "inkscape", "krita", "leftover"];

fn system() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(packages(&INSTALLED)));
    snapshot.insert(SyncPackages(packages(&OFFERED)));
    snapshot.insert(PackageGraph::default());

    // The helper refuses this build folder, so an AUR build takes paru's own path.
    snapshot.insert(AurRoute::Refused {
        kind: RouteKind::Linked,
        detail: "the home folder is a link".to_owned(),
    });
    snapshot
}

fn app(theme: Theme, language: Option<&str>) -> Fixture {
    app_with(theme, language, false)
}

/// The page on the fake database, with the dry run's sample AUR shown
/// when `aur`.
pub fn app_with(theme: Theme, language: Option<&str>, aur: bool) -> Fixture {
    app_with_aur(theme, language, aur, Arc::new(SampleAur))
}

pub fn app_with_aur(
    theme: Theme,
    language: Option<&str>,
    aur: bool,
    source: Arc<dyn AurSource>,
) -> Fixture {
    let (mut harness, runtime) = launch(theme, language, aur, source);
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

/// The app on the fake database after its first read, on its first page.
pub fn launch(
    theme: Theme,
    language: Option<&str>,
    aur: bool,
    source: Arc<dyn AurSource>,
) -> (Harness<'static, App>, Runtime) {
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
                        aur: source,
                        orders: Arc::new(toolbox_system::SampleAurOrders),
                        route: Arc::new(toolbox_system::SampleRoute),
                    }),
                    system_language,
                    preferences: preferences(aur),
                    ..Setup::default()
                },
            )
        });

    first_read(&mut harness);
    (harness, runtime)
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

pub fn open_package(harness: &mut Harness<'static, App>, package: &str, url: &str) {
    harness
        .get_by_role_and_label(Role::ListItem, package)
        .click();
    step_until(harness, "the details", |harness| {
        harness.query_by_label(url).is_some()
    });
    settle(harness);
}

/// Narrows the window until the list and its details no longer fit side by
/// side, so the details replace the list with Back.
pub fn narrow(harness: &mut Harness<'static, App>) {
    harness.set_size(egui::vec2(SMALLEST.0, TALL));
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

    harness
        .get_by_role_and_label(Role::ListItem, "inkscape")
        .click();
    harness.run();
    harness.get_by_label("Install inkscape").click();
    harness.run();

    assert!(harness.query_all_by_label("Installs with Apply").count() >= 1);
    assert!(queued_shown(harness));
}

#[test]
fn a_removal_names_what_goes_along_and_waits_in_the_queue() {
    let mut fixture = app(Theme::Dark, None);
    let harness = fixture.harness();
    open_package(harness, "gimp", "https://www.gimp.org/");

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
    assert!(harness.query_by_label("Take Back gimp").is_some());
    assert!(queued_shown(harness));
}

/// The back button leads the title's row, on the row's centre line, and
/// is not pressed against the window's top edge.
#[test]
fn the_back_button_leads_the_title_row() {
    let mut fixture = app(Theme::Dark, None);
    let harness = fixture.harness();
    open_package(harness, "gimp", "https://www.gimp.org/");

    narrow(harness);
    let back = harness
        .get_by_role_and_label(Role::Button, "Back to Packages")
        .rect();

    let title = harness.get_by_role_and_label(Role::Heading, "gimp").rect();

    assert!(
        back.right() <= title.left(),
        "the back button {back:?} is not left of the title {title:?}"
    );

    assert!(
        (back.center().y - title.center().y).abs() <= 2.0,
        "the back button {back:?} is off the title's row {title:?}"
    );
    assert!(back.top() >= 4.0, "the back button sits at {back:?}");
}

#[test]
fn a_dependency_opens_its_own_page_and_back_returns_to_the_first() {
    let mut fixture = app(Theme::Dark, None);
    let harness = fixture.harness();
    open_package(harness, "gimp", "https://www.gimp.org/");

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

    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(listed(harness, "gimp"));
}

#[test]
fn snapshots() {
    let mut results = SnapshotResults::new();

    let mut dark = app(Theme::Dark, None);
    let harness = dark.harness();
    results.add(harness.try_snapshot("packages/list_dark"));

    open_package(harness, "gimp", "https://www.gimp.org/");
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
    open_package(bright, "gimp", "https://www.gimp.org/");
    results.add(bright.try_snapshot("packages/detail_light"));

    let mut german = app(Theme::Dark, Some("de-DE"));
    let deutsch = german.harness();
    results.add(deutsch.try_snapshot("packages/list_german"));
    open_package(deutsch, "gimp", "https://www.gimp.org/");
    results.add(deutsch.try_snapshot("packages/detail_german"));
}

#[test]
fn at_the_narrowest_two_columns_the_list_and_details_do_not_overlap() {
    for language in [None, Some("de-DE")] {
        let mut fixture = app(Theme::Dark, language);
        let harness = fixture.harness();
        open_package(harness, "gimp", "https://www.gimp.org/");

        let width = narrowest_two_columns(harness);
        let line = column_line(harness);
        let rows: Vec<_> = harness
            .query_all_by_role(Role::ListItem)
            .map(|row| row.rect())
            .collect();

        assert!(
            rows.len() >= INSTALLED.len(),
            "{language:?}: at {width} the list shows {rows:?}"
        );

        for rect in rows {
            assert!(
                rect.right() <= line,
                "{language:?}: at {width} the row {rect:?} crosses the line at {line} into the details"
            );
        }
    }
}

/// The narrowest window that still lists the packages beside the open
/// details, found by halving the widths between the smallest and the usual
/// window.
fn narrowest_two_columns(harness: &mut Harness<'static, App>) -> f32 {
    let (mut narrow, mut wide) = (SMALLEST.0, WIDTH);
    for _ in 0..10_u32 {
        let middle = f32::midpoint(narrow, wide);
        harness.set_size(egui::vec2(middle, TALL));
        settle(harness);
        if listed(harness, "leftover") {
            wide = middle;
        } else {
            narrow = middle;
        }
    }

    harness.set_size(egui::vec2(wide, TALL));
    settle(harness);
    wide
}

/// The x of the line the columns draw between the list and the details,
/// the vertical line in the separator colour nearest left of the details'
/// heading. The details column starts a gap right of it.
fn column_line(harness: &Harness<'static, App>) -> f32 {
    let heading = harness.get_by_role_and_label(Role::Heading, "gimp").rect();
    let separator = toolbox_gui_kit::tokens::current(&harness.ctx).separator;
    toolbox_gui_kit::testing::lines_in(harness, separator)
        .iter()
        .filter(|[from, to]| (from.x - to.x).abs() < 0.01 && from.x < heading.left())
        .map(|[from, _]| from.x)
        .max_by(f32::total_cmp)
        .expect("the line between the columns")
}
