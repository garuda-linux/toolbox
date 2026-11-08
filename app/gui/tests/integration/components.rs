//! The Components page: sections, a click that queues and takes back, the
//! search, refusals, entries that wait and states that could not be read.

use std::collections::BTreeSet;
use std::fs;

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use tempfile::TempDir;
use tokio::runtime::Runtime;
use toolbox_core::domains::{Accounts, CurrentUser, Group, InstalledPackages, User};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui::catalogs::Catalogs;
use toolbox_gui::queue::Queue;
use toolbox_gui::{App, Setup, snapshot};

use crate::common::{HEIGHT, checkbox, packages, type_into, wheel, window};
use crate::discover::source_catalogs;
use crate::system::first_read;

const OWN_GROUP: &str = r#"
[[section]]
id = "groups"
title = "Groups"
icon = "computer"

[[section.entry]]
id = "own-group"
title = "Own group"
description = "The primary group of the test user."
check = { group = "nico" }
"#;

fn installed(names: &[&str]) -> InstalledPackages {
    InstalledPackages(packages(names))
}

/// The app on the Components page with `catalogs`, after the first read
/// of a system that looks like `system`.
fn app(catalogs: Catalogs, system: Snapshot, dry_run: bool) -> (Harness<'static, App>, Runtime) {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();

    let mut harness = window(HEIGHT).build_eframe(move |cc| {
        let probe = move || (system.clone(), Vec::new());
        let setup = Setup {
            queue: Queue::in_memory(ChangeSet::default()),
            snapshot: Some(snapshot::spawn(
                &handle,
                toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
                probe,
            )),
            catalogs,
            dry_run,
            ..Setup::default()
        };

        App::new(cc, setup)
    });

    first_read(&mut harness);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Num3);
    harness.run();
    (harness, runtime)
}

pub fn components(system: Snapshot) -> (Harness<'static, App>, Runtime) {
    app(source_catalogs(), system, false)
}

pub fn with_pipewire() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(installed(&["pipewire-support"]));
    snapshot
}

#[test]
fn sections_show_and_a_click_queues_and_takes_back() {
    let (mut harness, _runtime) = components(with_pipewire());
    assert!(harness.query_by_label("1/8").is_some());

    checkbox(&harness, "PulseAudio").click();
    harness.run();
    assert!(harness.query_by_label("change queued").is_some());
    assert!(harness.query_by_label("Turns on with Apply").is_some());

    checkbox(&harness, "PulseAudio").click();
    harness.run();
    assert!(harness.query_by_label("change queued").is_none());
}

#[test]
fn an_entry_waits_for_what_it_needs() {
    let mut system = Snapshot::default();
    system.insert(installed(&[]));
    let (mut harness, _runtime) = components(system);

    // Rows below the window are not laid out, so the search brings it up.
    type_into(&mut harness, "Search this page", "jamesdsp");

    assert!(
        checkbox(&harness, "JamesDSP")
            .accesskit_node()
            .is_disabled()
    );
    assert!(
        harness
            .query_all_by_label("Needs PipeWire")
            .next()
            .is_some()
    );
}

#[test]
fn the_search_keeps_matching_entries_and_says_when_none_match() {
    let (mut harness, _runtime) = components(with_pipewire());

    type_into(&mut harness, "Search this page", "jamesdsp");
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::CheckBox, "JamesDSP")
            .is_some()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::CheckBox, "PulseAudio")
            .is_none()
    );

    type_into(&mut harness, "Search this page", "zzz");
    harness.run();
    assert!(
        harness
            .query_by_label("Nothing matches the search")
            .is_some()
    );

    harness.get_by_label("Clear search").click();
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::CheckBox, "PulseAudio")
            .is_some()
    );
}

#[test]
fn entries_without_a_read_state_are_told_once() {
    let (harness, _runtime) = components(Snapshot::default());

    assert!(
        harness
            .query_by_label_contains(
                "could not be read, so their switches may show the wrong state"
            )
            .is_some()
    );
    assert!(
        harness.query_by_label("State not known.").is_none(),
        "a row repeats the page banner"
    );
    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Read again")
            .is_some()
    );

    let pulse = harness.get_by_role_and_label(Role::CheckBox, "PulseAudio");
    assert_eq!(
        pulse.accesskit_node().description().as_deref(),
        Some("State not known."),
        "a screen reader still hears it per row"
    );
}

#[test]
fn a_refused_click_says_why_until_dismissed() {
    let folder = TempDir::new().unwrap();
    fs::write(folder.path().join("components.toml"), OWN_GROUP).unwrap();
    let user = "nico".parse().unwrap();
    let mut system = Snapshot::default();
    system.insert(Accounts {
        users: vec![User {
            name: "nico".parse().unwrap(),
            uid: 1000,
            gid: 1000,
            gecos: String::new(),
            home: "/home/nico".to_owned(),
            shell: "/usr/bin/bash".to_owned(),
        }],
        groups: vec![Group {
            name: "nico".parse().unwrap(),
            gid: 1000,
            members: BTreeSet::new(),
        }],
    });
    system.insert(CurrentUser(user));
    let (mut harness, _runtime) = app(Catalogs::load(folder.path()), system, false);

    checkbox(&harness, "Own group").click();
    harness.run();
    assert!(
        harness
            .query_by_label("nico is the primary group of a user, so it stays.")
            .is_some()
    );

    harness.get_by_label("Dismiss").click();
    harness.run();
    assert!(
        harness
            .query_by_label("nico is the primary group of a user, so it stays.")
            .is_none()
    );
}

#[test]
fn a_dry_run_says_so_in_the_status_bar() {
    let (harness, _runtime) = app(Catalogs::default(), Snapshot::default(), true);

    assert!(
        harness
            .query_by_label("Test mode: nothing is changed.")
            .is_some()
    );
}

#[test]
fn the_page_header_stays_while_the_page_scrolls() {
    let (mut harness, _runtime) = components(with_pipewire());
    let search = |window: &Harness<'static, App>| {
        window
            .get_by_role_and_label(Role::TextInput, "Search this page")
            .rect()
    };

    let before = search(&harness);
    let row = checkbox(&harness, "PulseAudio").rect();

    wheel(&harness, row.center(), egui::vec2(0.0, -300.0));
    harness.run();

    let after = checkbox(&harness, "PulseAudio").rect();
    assert!(after.top() < row.top() - 100.0, "the page scrolled");
    assert_eq!(search(&harness), before);
}

#[test]
fn during_a_search_a_chip_counts_its_matches() {
    let (mut harness, _runtime) = components(with_pipewire());
    let audio = |shown: &Harness<'static, App>| {
        shown
            .get_all_by_role(Role::Button)
            .filter_map(|node| node.accesskit_node().label())
            .find(|label| label.starts_with("Audio "))
            .unwrap()
    };

    assert_eq!(audio(&harness), "Audio 1/8");

    type_into(&mut harness, "Search this page", "jamesdsp");
    assert_eq!(audio(&harness), "Audio 0/1");
}
