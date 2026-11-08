//! System Tweaks: the system-wide choices as sections before the settings
//! catalog, picked with dropdowns and switches, under one search.

use std::collections::{BTreeMap, BTreeSet};

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use tokio::runtime::Runtime;
use toolbox_core::domains::{
    AvailableShells, CurrentDns, CurrentShells, CurrentWifiBackend, GarudaUpdateConfig, HblockState,
};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui::queue::Queue;
use toolbox_gui::{App, Setup, snapshot};
use toolbox_plan::{DnsProvider, GarudaUpdateFlag, ShellPath, UserName, WifiBackend};

use crate::common::{TALL, type_into, window};
use crate::discover::source_catalogs;
use crate::system::first_read;

fn user() -> UserName {
    UserName::new("tester").unwrap()
}

fn shell(path: &str) -> ShellPath {
    ShellPath::new(path).unwrap()
}

/// bash is the login shell, DNS from the network, `wpa_supplicant`, no
/// hblock, garuda-update with the AUR option on.
pub fn system() -> Snapshot {
    let mut snapshot = crate::home::host(false, false);
    snapshot.insert(AvailableShells(vec![
        shell("/usr/bin/bash"),
        shell("/usr/bin/zsh"),
    ]));
    snapshot.insert(CurrentShells(BTreeMap::from([(
        user(),
        shell("/usr/bin/bash"),
    )])));
    snapshot.insert(CurrentDns(DnsProvider::Default));
    snapshot.insert(CurrentWifiBackend(WifiBackend::WpaSupplicant));
    snapshot.insert(HblockState(false));
    snapshot.insert(GarudaUpdateConfig(BTreeSet::from([
        GarudaUpdateFlag::UpdateAur,
    ])));
    snapshot
}

/// The app on System Tweaks after the first read of `system`.
pub fn tweaks(system: Snapshot) -> (Harness<'static, App>, Runtime) {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();
    let catalogs = source_catalogs();

    let mut harness = window(TALL).build_eframe(move |cc| {
        let probe = move || (system.clone(), Vec::new());
        App::new(
            cc,
            Setup {
                queue: Queue::in_memory(ChangeSet::default()),
                snapshot: Some(snapshot::spawn(
                    &handle,
                    toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
                    probe,
                )),
                catalogs,
                ..Setup::default()
            },
        )
    });

    first_read(&mut harness);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Num8);
    harness.run();
    (harness, runtime)
}

fn pick(harness: &mut Harness<'static, App>, dropdown: &str, option: &str) {
    harness
        .get_by_role_and_label(Role::ComboBox, dropdown)
        .click();
    harness.run();
    harness.get_by_label(option).click();
    harness.run();
}

#[test]
fn the_choices_lead_the_catalog() {
    let (harness, _runtime) = tweaks(system());

    assert_eq!(
        harness.query_all_by_label("Shell and network").count(),
        2,
        "the heading and the list it heads, the chip adds its count"
    );
    assert_eq!(
        harness.query_all_by_label("Garuda Update").count(),
        2,
        "the heading and the list it heads"
    );
    assert!(harness.query_by_label("1/4").is_some());
    assert!(
        harness
            .query_by_role_and_label(Role::CheckBox, "Profile-sync-daemon")
            .is_some()
    );
}

#[test]
fn a_dropdown_queues_another_option_and_not_the_current_one() {
    let (mut harness, _runtime) = tweaks(system());

    pick(&mut harness, "DNS server", "From your network (default)");
    assert!(harness.query_by_label("change queued").is_none());

    pick(&mut harness, "DNS server", "Quad9 (9.9.9.9)");
    assert!(harness.query_by_label("Changes with Apply").is_some());
    assert!(harness.query_by_label("change queued").is_some());

    pick(&mut harness, "Login shell", "zsh");
    assert!(harness.query_by_label("changes queued").is_some());
}

#[test]
fn a_switch_queues_hblock_and_a_garuda_update_option() {
    let (mut harness, _runtime) = tweaks(system());

    harness
        .get_by_role_and_label(Role::CheckBox, "Ad blocking")
        .click();
    harness.run();
    assert!(harness.query_by_label("Turns on with Apply").is_some());

    // The list builds only the rows near the view, so the search brings
    // the row up instead of a scroll.
    type_into(&mut harness, "Search this page", "AUR packages");
    harness
        .get_by_role_and_label(Role::CheckBox, "Update AUR packages")
        .click();
    harness.run();
    assert!(harness.query_by_label("Turns off with Apply").is_some());
}

#[test]
fn the_search_and_chips_cover_the_choices_too() {
    let (mut harness, _runtime) = tweaks(system());

    type_into(&mut harness, "Search this page", "iwd");
    assert!(
        harness
            .query_by_role_and_label(Role::ComboBox, "Wi-Fi backend")
            .is_some()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::ComboBox, "Login shell")
            .is_none()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::CheckBox, "Profile-sync-daemon")
            .is_none()
    );
}

#[test]
fn a_chip_keeps_the_garuda_update_options_alone() {
    let (mut harness, _runtime) = tweaks(system());

    harness
        .get_by_role_and_label(Role::Button, "Garuda Update 1/4")
        .click();
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::ComboBox, "DNS server")
            .is_none()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::CheckBox, "Skip disk space check")
            .is_some()
    );
}

#[test]
fn choices_the_system_did_not_report_wait() {
    let (harness, _runtime) = tweaks(Snapshot::default());

    assert!(
        harness
            .get_by_role_and_label(Role::ComboBox, "DNS server")
            .accesskit_node()
            .is_disabled()
    );
    assert!(harness.query_by_label("Garuda Update").is_none());
}

#[test]
fn a_search_marks_its_match_in_the_lead_rows_too() {
    let (mut harness, _runtime) = tweaks(system());
    type_into(&mut harness, "Search this page", "dns");

    let wash = toolbox_gui_kit::tokens::current(&harness.ctx).accent_soft;
    let title =
        toolbox_gui_kit::testing::painted_text(&harness, "DNS server").expect("the DNS row");

    let marked: Vec<&str> = title
        .galley
        .job
        .sections
        .iter()
        .filter(|section| section.format.background == wash)
        .filter_map(|section| {
            let range = &section.byte_range;
            title.galley.job.text.get(range.start.0..range.end.0)
        })
        .collect();

    assert_eq!(marked, ["DNS"]);
}

#[test]
fn during_a_search_the_garuda_update_chip_counts_its_matches() {
    let (mut harness, _runtime) = tweaks(system());

    type_into(&mut harness, "Search this page", "aur");

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Garuda Update 1/1")
            .is_some()
    );
}

#[test]
fn during_a_search_a_section_heading_counts_its_matches() {
    let (mut harness, _runtime) = tweaks(system());
    type_into(&mut harness, "Search this page", "aur");

    let heading = harness
        .get_by_role_and_label(Role::Heading, "Garuda Update")
        .rect();

    let beside = harness
        .query_all_by_label("1/1")
        .any(|count| (count.rect().center().y - heading.center().y).abs() < heading.height());

    assert!(beside, "the Garuda Update heading does not count the match");
}
