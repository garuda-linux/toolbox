//! Pictures of the shell, compared with the ones in `tests/snapshots/`.
//! `UPDATE_SNAPSHOTS=1` writes new ones after a wanted change.

mod pages;

use egui::accesskit::Role;
use egui::{Key, Modifiers, Theme};
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use egui_kittest::{Harness, SnapshotResults};
use toolbox_core::domains::{Hblock, Packages};
use toolbox_core::{ChangeSet, Single, Snapshot, Toggle};
use toolbox_gui::queue::Queue;
use toolbox_gui::{App, Setup};

use crate::common::{HEIGHT, installing, launch_in, window};

fn app(theme: Theme, setup: Setup) -> Harness<'static, App> {
    launch_in(window(HEIGHT).with_theme(theme), setup)
}

#[test]
fn shell() {
    let mut results = SnapshotResults::new();

    let mut dark = app(Theme::Dark, Setup::default());
    results.add(dark.try_snapshot("shell/home_dark"));

    dark.get_by_label("Collapse sidebar").click();
    dark.run();
    results.add(dark.try_snapshot("shell/collapsed_dark"));

    let mut tools = app(Theme::Dark, Setup::default());
    tools.key_press_modifiers(Modifiers::COMMAND, Key::J);
    tools.key_press_modifiers(Modifiers::COMMAND, Key::K);
    tools.run();
    results.add(tools.try_snapshot("shell/console_and_palette_dark"));

    let mut light = app(Theme::Light, Setup::default());
    results.add(light.try_snapshot("shell/home_light"));

    let mut german = app(
        Theme::Dark,
        Setup {
            system_language: "de-DE".parse().unwrap(),
            ..Setup::default()
        },
    );

    results.add(german.try_snapshot("shell/home_german"));
}

#[test]
fn changes_panel() {
    let mut results = SnapshotResults::new();
    let mut changes = ChangeSet::default();
    let snapshot = Snapshot::default();
    changes.want::<Packages>("firefox".parse().unwrap(), Toggle::On, &snapshot);
    changes.want::<Packages>("steam".parse().unwrap(), Toggle::Off, &snapshot);
    changes.want::<Hblock>(Single, Toggle::On, &snapshot);

    let mut harness = app(
        Theme::Dark,
        Setup {
            queue: Queue::in_memory(changes),
            ..Setup::default()
        },
    );

    results.add(harness.try_snapshot("changes/pill_dark"));

    harness.get_by_label("Review").click();
    harness.run();
    results.add(harness.try_snapshot("changes/popover_dark"));

    harness.get_by_label("Remove all…").click();
    harness.run();
    results.add(harness.try_snapshot("changes/question_dark"));
}

#[test]
fn console_after_apply() {
    let (mut harness, _runtime) =
        crate::console::installing_app(crate::console::succeeding_script(), crate::console::JAVA);

    harness.get_by_label("Apply").click();
    harness.step();
    crate::console::wait_for(&mut harness, "All changes applied");
    crate::system::step_until(&mut harness, "the queue to empty", |harness| {
        harness.query_by_label("Review").is_none()
    });

    // The read after the run finds java-runtime, so nothing is left to apply.
    assert!(harness.query_by_label("Apply").is_none());
    crate::console::unfocus(&mut harness);
    harness.snapshot("console/applied_dark");
}

#[test]
fn console_with_nothing_to_do() {
    let (mut harness, _runtime) = crate::console::app(crate::console::idle_script(), &["firefox"]);
    harness.get_by_label("Apply").click();
    harness.step();
    crate::console::wait_for(&mut harness, "Nothing needed to change");

    crate::console::unfocus(&mut harness);
    harness.snapshot("console/nothing_dark");
}

#[test]
fn status_bar_with_failed_probes() {
    let (mut harness, _runtime, answer, _reads) = crate::system::app();
    let failed = |probe| toolbox_system::ProbeIssue {
        probe,
        error: toolbox_system::ProbeError::Failed("no bus".to_owned()),
    };

    let issues = vec![
        failed(toolbox_system::Probe::Units),
        failed(toolbox_system::Probe::Accounts),
    ];

    answer.send((Snapshot::default(), issues)).unwrap();

    crate::system::step_until(
        &mut harness,
        "the failure notice",
        crate::system::shows(
            "Some parts of the system could not be read: system services, users and groups",
        ),
    );
    harness.run();

    harness.snapshot("status/failed_dark");
}

#[test]
fn components_page() {
    let system = read_in_full(crate::components::with_pipewire());
    let (mut harness, _runtime) = crate::components::components(system);
    no_unread_banner(&harness);
    crate::common::checkbox(&harness, "PulseAudio").click();
    harness.run();

    let mut results = SnapshotResults::new();
    results.add(harness.try_snapshot("components/page_dark"));

    harness.ctx.set_theme(Theme::Light);
    harness.run();
    results.add(harness.try_snapshot("components/page_light"));
}

#[test]
fn gaming_page() {
    let (mut harness, _runtime) = crate::gaming::gaming();

    // The pictures load from their files on other threads.
    for _ in 0_u8..20 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        harness.step();
    }

    harness.run();

    harness.snapshot("gaming/page_dark");
}

#[test]
fn kernels_page() {
    let (mut harness, _runtime) = crate::kernels::kernels(crate::kernels::system(true));
    harness
        .get_by_role_and_label(Role::CheckBox, "linux-zen")
        .click();
    harness.run();

    harness.snapshot("kernels/page_dark");
}

#[test]
fn home_page() {
    use std::time::{Duration, SystemTime};

    use toolbox_system::{HealthIssue, HealthLevel};

    let mut status = crate::home::quiet();
    status.updates = Ok(vec![
        crate::home::update("linux"),
        crate::home::update("mesa"),
    ]);
    status.aur_updates = Ok(vec![crate::home::update("paru-bin")]);
    status.last_update = Ok(Some(SystemTime::now() - Duration::from_hours(24 * 16)));
    status.health = Ok(vec![HealthIssue {
        level: HealthLevel::Low,
        message: "The keyring is older than three months.".to_owned(),
        fix_available: true,
    }]);

    let (mut harness, _runtime, _power) = crate::home::home(crate::home::host(false, true), status);
    harness.snapshot("home/page_dark");
}

#[test]
fn tweaks_page() {
    let (mut harness, _runtime) = crate::tweaks::tweaks(read_in_full(crate::tweaks::system()));
    no_unread_banner(&harness);
    harness
        .get_by_role_and_label(Role::ComboBox, "DNS server")
        .click();
    harness.run();

    harness.snapshot("tweaks/dropdown_dark");
}

#[test]
fn services_page() {
    let (mut harness, _runtime) = crate::services::services();

    crate::services::open_menu(&mut harness, "sshd.service");

    harness.snapshot("services/menu_dark");
}

#[test]
fn discover_page() {
    use crate::discover::{Table, chip, pictured, source_catalogs, source_tree, system};

    // A 4×4 JPEG XL picture of one colour, as Arch's AppStream cache has.
    const PINK: [u8; 35] = [
        0xff, 0x0a, 0x18, 0x10, 0x10, 0x09, 0x08, 0x00, 0x01, 0x00, 0x5c, 0x00, 0x4b, 0x18, 0x8b,
        0x15, 0xc2, 0x09, 0x32, 0x6f, 0x54, 0xfc, 0x07, 0x50, 0x0f, 0x00, 0x00, 0xd8, 0x47, 0x2a,
        0x00, 0x3c, 0x10, 0x00, 0x00,
    ];

    let dir = tempfile::TempDir::new().unwrap();
    let jxl = dir.path().join("aerc.jxl");
    std::fs::write(&jxl, PINK).unwrap();
    let table = Table(vec![
        ("firefox", source_tree().join("icons/gaming/bottles.svg")),
        ("aerc", jxl),
    ]);

    let (mut harness, _runtime) = pictured(source_catalogs(), system(), Some(table));
    chip(&mut harness, "Internet");
    harness
        .get_by_role_and_label(Role::CheckBox, "Firefox")
        .click();

    // The answers and the decoded pictures come from other threads.
    for _ in 0_u8..30 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        harness.step();
    }

    harness.run();

    harness.snapshot("discover/page_dark");
}

#[test]
fn settings_page() {
    let (mut harness, _store) = crate::settings::settings();

    harness.snapshot("settings/page_dark");
}

#[test]
fn about_page() {
    let mut harness = app(Theme::Dark, Setup::default());
    harness.get_by_label("About").click();
    harness.run();

    harness.snapshot("about/page_dark");
}

#[test]
fn frost_in_light() {
    let mut results = SnapshotResults::new();
    let setup = || Setup {
        queue: Queue::in_memory(installing(&["firefox"])),
        ..Setup::default()
    };

    let mut dialog = app(Theme::Light, setup());
    dialog.get_by_label("Review").click();
    dialog.run();
    dialog.get_by_label("Remove all…").click();
    dialog.run();
    results.add(dialog.try_snapshot("frost/question_light"));

    let mut console = app(Theme::Light, setup());
    console.key_press_modifiers(Modifiers::COMMAND, Key::J);
    console.run();
    results.add(console.try_snapshot("frost/console_light"));
}

/// `system` as a full read sees it. A part left out would show the
/// could-not-be-read banner, which the pictures of a normal page should not.
fn read_in_full(mut system: Snapshot) -> Snapshot {
    use toolbox_core::domains::{
        Accounts, CurrentUser, EnabledUnits, EnabledUserUnits, InstalledPackages, User,
        UserConfigStates,
    };

    let tester: toolbox_plan::UserName = "tester".parse().unwrap();
    if system.part::<InstalledPackages>().is_none() {
        system.insert(InstalledPackages(std::collections::BTreeSet::new()));
    }

    system.insert(EnabledUnits(std::collections::BTreeSet::new()));
    system.insert(EnabledUserUnits(std::collections::BTreeSet::new()));
    system.insert(Accounts {
        users: vec![User {
            name: tester.clone(),
            uid: 1000,
            gid: 1000,
            gecos: String::new(),
            home: "/home/tester".to_owned(),
            shell: "/usr/bin/bash".to_owned(),
        }],
        groups: Vec::new(),
    });
    system.insert(CurrentUser(tester));
    let edits = crate::discover::source_catalogs().config_edits();
    system.insert(UserConfigStates(
        edits.into_iter().map(|edit| (edit, false)).collect(),
    ));
    system
}

fn no_unread_banner(harness: &Harness<'static, App>) {
    assert!(
        harness
            .query_by_label_contains("could not be read")
            .is_none(),
        "the fixture leaves parts of the system unread: {:?}",
        harness
            .query_by_label_contains("could not be read")
            .map(|node| format!("{:?}", node.accesskit_node()))
    );
}
