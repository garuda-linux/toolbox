//! The Packages page's AUR chip with the dry run's sample AUR: the hint
//! while the setting is off, the list, a detail page and the question
//! before a build, in dark, light, German and the smallest window.

use std::sync::Arc;

use egui::Theme;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use egui_kittest::{Harness, SnapshotResults};
use toolbox_aur::{AurError, AurSource};
use toolbox_core::features::packages::aur::AurIndex;
use toolbox_gui::App;
use toolbox_gui::packages::SampleAur;

use crate::common::{SMALLEST, TALL, WIDTH};
use crate::packages::{app_with, app_with_aur, listed};
use crate::system::{settle, step_until};

/// The sample AUR whose last refresh failed.
struct Stale;

impl AurSource for Stale {
    fn index(&self) -> Result<AurIndex, AurError> {
        SampleAur.index().map(|index| AurIndex {
            stale: true,
            ..index
        })
    }
}

fn aur_list(harness: &mut Harness<'static, App>) {
    harness.get_by_role_and_label(Role::Button, "AUR").click();
    step_until(harness, "the AUR list", |harness| {
        listed(harness, "paru-bin")
    });
    settle(harness);
}

fn open_paru(harness: &mut Harness<'static, App>) {
    harness
        .get_by_role_and_label(Role::ListItem, "paru-bin")
        .click();
    step_until(harness, "the AUR details", |harness| {
        harness
            .query_by_label("https://aur.archlinux.org/packages/paru-bin")
            .is_some()
    });
    settle(harness);
}

/// `install` is the button's word in the language shown.
fn ask_install(harness: &mut Harness<'static, App>, install: &str) {
    harness.get_by_label(&format!("{install} yay-bin")).click();
    settle(harness);
}

fn narrow(harness: &mut Harness<'static, App>) {
    harness.set_size(egui::vec2(SMALLEST.0, TALL));
    settle(harness);
}

#[test]
fn an_aur_build_asks_first_and_waits_in_the_queue() {
    let mut fixture = app_with(Theme::Dark, None, true);
    let harness = fixture.harness();
    aur_list(harness);

    ask_install(harness, "Install");
    assert!(
        harness
            .query_by_label("Built from a community script on your computer. paru shows the build script in the Console before it builds.")
            .is_some()
    );

    harness
        .get_by_role_and_label(Role::Button, "Install")
        .click();
    harness.run();

    assert!(harness.query_by_label("Builds with Apply").is_some());
    assert!(harness.query_by_label("change queued").is_some());
}

#[test]
fn the_aur_chip_hides_while_the_setting_is_off() {
    let mut fixture = app_with(Theme::Dark, None, false);
    let harness = fixture.harness();

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "AUR")
            .is_none()
    );
    assert!(harness.query_by_label("AUR packages are hidden.").is_some());
}

#[test]
fn snapshots() {
    let mut results = SnapshotResults::new();

    let mut off = app_with(Theme::Dark, None, false);
    let hidden = off.harness();
    narrow(hidden);
    results.add(hidden.try_snapshot("packages/aur_off_narrow"));

    for (theme, language, name, install) in [
        (Theme::Dark, None, "dark", "Install"),
        (Theme::Light, None, "light", "Install"),
        (Theme::Dark, Some("de-DE"), "german", "Installieren"),
    ] {
        let mut fixture = app_with(theme, language, true);
        let harness = fixture.harness();
        aur_list(harness);
        results.add(harness.try_snapshot(format!("packages/aur_list_{name}")));

        ask_install(harness, install);
        results.add(harness.try_snapshot(format!("packages/aur_install_{name}")));
        harness.key_press(egui::Key::Escape);
        settle(harness);

        open_paru(harness);
        results.add(harness.try_snapshot(format!("packages/aur_detail_{name}")));
    }

    // The flag mark's German words, the longest on the page.
    let mut flagged = app_with(Theme::Dark, Some("de-DE"), true);
    let deutsch = flagged.harness();
    aur_list(deutsch);
    deutsch
        .get_by_role_and_label(Role::ListItem, "toolbox-sample-git")
        .click();
    step_until(deutsch, "the flagged details", |harness| {
        harness.query_by_label("Veraltet seit").is_some()
            && harness
                .query_by_label("https://aur.archlinux.org/packages/toolbox-sample-git")
                .is_some()
    });
    settle(deutsch);
    results.add(deutsch.try_snapshot("packages/aur_detail_flagged_german"));

    let mut small = app_with(Theme::Dark, None, true);
    let harness = small.harness();
    narrow(harness);
    aur_list(harness);
    results.add(harness.try_snapshot("packages/aur_list_narrow"));
    ask_install(harness, "Install");
    results.add(harness.try_snapshot("packages/aur_install_narrow"));
    harness.key_press(egui::Key::Escape);
    settle(harness);
    open_paru(harness);
    results.add(harness.try_snapshot("packages/aur_detail_narrow"));

    harness.set_size(egui::vec2(WIDTH, TALL));
    settle(harness);
}

fn line_to_list(harness: &Harness<'static, App>, line: &str, first: &str) -> f32 {
    let line = harness.get_by_label_contains(line).rect();
    let row = harness.get_by_role_and_label(Role::ListItem, first).rect();
    row.top() - line.bottom()
}

#[test]
fn the_line_under_the_chips_keeps_one_gap_to_the_list_in_either_view() {
    let mut off = app_with(Theme::Dark, None, false);
    let hidden = line_to_list(off.harness(), "AUR packages are hidden.", "babl");

    let mut on = app_with(Theme::Dark, None, true);
    let harness = on.harness();
    aur_list(harness);
    let noted = line_to_list(harness, "List of ", "paru-bin");

    assert!(
        (hidden - noted).abs() < 1.0,
        "{hidden} under the hidden AUR's line, {noted} under the AUR's note"
    );
}

#[test]
fn the_line_under_the_chips_sits_at_one_height_in_either_view() {
    let mut off = app_with(Theme::Dark, None, false);
    let hidden = off
        .harness()
        .get_by_label_contains("AUR packages are hidden.")
        .rect();

    let mut on = app_with(Theme::Dark, None, true);
    let harness = on.harness();
    aur_list(harness);
    let noted = harness.get_by_label_contains("List of ").rect();

    assert!(
        (hidden.center().y - noted.center().y).abs() < 1.0,
        "the hidden AUR's line at {hidden:?}, the AUR's note at {noted:?}"
    );
}

#[test]
fn a_list_that_could_not_be_refreshed_says_so_beside_its_date() {
    let mut fixture = app_with_aur(Theme::Dark, None, true, Arc::new(Stale));
    let harness = fixture.harness();
    aur_list(harness);

    assert!(
        harness
            .query_by_label_contains("could not be updated")
            .is_some()
    );
    harness.snapshot("packages/aur_stale_dark");
}

#[test]
fn a_fresh_list_shows_no_warning() {
    let mut fixture = app_with(Theme::Dark, None, true);
    let harness = fixture.harness();
    aur_list(harness);

    assert!(
        harness
            .query_by_label_contains("could not be updated")
            .is_none()
    );
}
