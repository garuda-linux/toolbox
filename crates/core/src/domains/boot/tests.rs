use toolbox_plan::{DefaultEntry, GrubKey, GrubSetting, GrubState, GrubTimeout, KernelParameters};

use super::{CurrentGrub, GrubDefaults, PlymouthState, PlymouthThemes};
use crate::domain::{Domain as _, Summary, Toggle};
use crate::domains::Packages;
use crate::test_support::{
    commands, failed_at, outcomes, package, plymouth_theme, restored, stopped_at, views,
};
use crate::{ChangeSet, Single, Snapshot, domains, plan};

const DEFAULTS: &str = "GRUB_DEFAULT=0\nGRUB_TIMEOUT=5\nGRUB_CMDLINE_LINUX_DEFAULT=\"quiet\"\n";
const REBUILD: &str = "dracut-rebuild, or mkinitcpio -P where dracut-rebuild is not installed";

fn snapshot(defaults: &str, current: &str) -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(CurrentGrub(GrubState::read(defaults)));
    snapshot.insert(PlymouthState {
        themes: vec![plymouth_theme("bgrt"), plymouth_theme("spinner")],
        current: Some(plymouth_theme(current)),
    });
    snapshot
}

fn timeout(seconds: i64) -> GrubSetting {
    GrubSetting::Timeout(GrubTimeout::new(seconds).unwrap())
}

fn parameters(words: &str) -> GrubSetting {
    GrubSetting::KernelParameters(KernelParameters::parse(words).unwrap())
}

#[test]
fn a_wish_the_file_already_fulfils_drops_out() {
    let state = snapshot(DEFAULTS, "bgrt");
    let mut changes = ChangeSet::default();

    GrubDefaults::want(&mut changes, timeout(5), &state);
    GrubDefaults::want(&mut changes, parameters("quiet"), &state);
    GrubDefaults::want(&mut changes, GrubSetting::Submenus(true), &state);
    GrubDefaults::want(&mut changes, GrubSetting::OsProber(false), &state);
    GrubDefaults::want(&mut changes, GrubSetting::Theme(None), &state);
    changes.want::<PlymouthThemes>(Single, plymouth_theme("bgrt"), &state);

    assert!(changes.is_empty());
}

#[test]
fn a_second_wish_for_a_variable_replaces_the_first_and_can_take_it_back() {
    let state = snapshot(DEFAULTS, "bgrt");
    let mut changes = ChangeSet::default();

    GrubDefaults::want(&mut changes, timeout(10), &state);
    GrubDefaults::want(&mut changes, timeout(20), &state);
    assert_eq!(changes.len(), 1);
    assert_eq!(
        GrubDefaults::queued(&changes, GrubKey::Timeout),
        Some(timeout(20))
    );

    GrubDefaults::want(&mut changes, timeout(5), &state);
    assert!(changes.is_empty());
}

#[test]
fn many_variables_are_one_edit_and_one_generation() {
    let state = snapshot(DEFAULTS, "bgrt");
    let mut changes = ChangeSet::default();

    GrubDefaults::want(&mut changes, parameters("quiet splash"), &state);
    GrubDefaults::want(&mut changes, timeout(3), &state);
    GrubDefaults::want(
        &mut changes,
        GrubSetting::Default(DefaultEntry::Saved),
        &state,
    );
    GrubDefaults::want(&mut changes, GrubSetting::SaveDefault(true), &state);

    assert_eq!(
        commands(&plan(&changes)),
        [
            "set GRUB_DEFAULT=\"saved\", GRUB_TIMEOUT=\"3\", GRUB_CMDLINE_LINUX_DEFAULT=\"quiet splash\", GRUB_SAVEDEFAULT=\"true\" in /etc/default/grub",
            "grub-mkconfig -o /boot/grub/grub.cfg",
        ]
    );
}

#[test]
fn packages_come_first_and_the_plymouth_theme_last() {
    let state = snapshot(DEFAULTS, "bgrt");
    let mut changes = ChangeSet::default();

    changes.want::<PlymouthThemes>(Single, plymouth_theme("spinner"), &state);
    GrubDefaults::want(&mut changes, timeout(3), &state);
    changes.want::<Packages>(package("plymouth"), Toggle::On, &state);

    assert_eq!(
        commands(&plan(&changes)),
        [
            "pacman -S --needed --noconfirm -- plymouth",
            "set GRUB_TIMEOUT=\"3\" in /etc/default/grub",
            "grub-mkconfig -o /boot/grub/grub.cfg",
            "plymouth-set-default-theme spinner",
            REBUILD,
        ]
    );
}

#[test]
fn only_a_theme_change_generates_no_menu() {
    let state = snapshot(DEFAULTS, "bgrt");
    let mut changes = ChangeSet::default();

    changes.want::<PlymouthThemes>(Single, plymouth_theme("spinner"), &state);

    assert_eq!(
        commands(&plan(&changes)),
        ["plymouth-set-default-theme spinner", REBUILD]
    );
}

#[test]
fn the_review_page_gets_the_sentence_and_the_new_value() {
    let state = snapshot(DEFAULTS, "bgrt");
    let mut changes = ChangeSet::default();

    GrubDefaults::want(&mut changes, timeout(3), &state);
    GrubDefaults::want(&mut changes, GrubSetting::OsProber(true), &state);
    GrubDefaults::want(&mut changes, GrubSetting::Submenus(false), &state);
    changes.want::<PlymouthThemes>(Single, plymouth_theme("spinner"), &state);

    assert_eq!(
        views(&changes),
        [
            (
                "grub-settings",
                "timeout".to_owned(),
                Summary::of("grub-timeout", "3")
            ),
            (
                "grub-settings",
                "submenus".to_owned(),
                Summary::plain("grub-submenus-off")
            ),
            (
                "grub-settings",
                "os-prober".to_owned(),
                Summary::plain("grub-os-prober-on")
            ),
            (
                "plymouth-theme",
                "plymouth-theme".to_owned(),
                Summary::of("plymouth-theme", "spinner")
            ),
        ]
    );
}

#[test]
fn applied_changes_leave_the_queue_with_the_next_snapshot() {
    let mut changes = ChangeSet::default();
    let before = snapshot(DEFAULTS, "bgrt");

    GrubDefaults::want(&mut changes, timeout(3), &before);
    changes.want::<PlymouthThemes>(Single, plymouth_theme("spinner"), &before);
    assert_eq!(changes.len(), 2);

    changes.reconcile(&snapshot("GRUB_TIMEOUT=3\n", "spinner"));
    assert!(changes.is_empty());
}

/// Without the file nothing can be compared, so a restored wish waits
/// for the probe instead of being dropped.
#[test]
fn queued_boot_changes_survive_a_save_and_restore() {
    let empty = Snapshot::default();
    let mut changes = ChangeSet::default();

    GrubDefaults::want(&mut changes, timeout(3), &empty);
    GrubDefaults::want(
        &mut changes,
        parameters("quiet splash mitigations=off"),
        &empty,
    );
    GrubDefaults::want(
        &mut changes,
        GrubSetting::Default(DefaultEntry::Saved),
        &empty,
    );
    changes.want::<PlymouthThemes>(Single, plymouth_theme("spinner"), &empty);

    let (restored, problems) = restored(&changes, &empty);

    assert_eq!(problems, []);
    assert_eq!(restored.views(), changes.views());
    assert_eq!(restored.len(), 4);
}

/// A queue file is plain JSON in the user's home. What it holds is
/// checked like a request on the wire.
#[test]
fn a_tampered_queue_file_cannot_smuggle_a_value_in() {
    let saved = r#"{"version":VERSION,"changes":[
        {"domain":"grub-settings","item":"kernel-parameters","wish":{"key":"kernel-parameters","value":["a\"; reboot; \""]}},
        {"domain":"grub-settings","item":"timeout","wish":{"key":"timeout","value":301}},
        {"domain":"grub-settings","item":"timeout","wish":{"key":"theme","value":null}},
        {"domain":"plymouth-theme","item":null,"wish":"--reset"}
    ]}"#
    .replace("VERSION", &crate::SAVED_VERSION.to_string());

    let mut restored = ChangeSet::default();
    let state = snapshot(DEFAULTS, "bgrt");

    let problems = domains::all().unwrap().restore(
        &mut restored,
        &serde_json::from_str(&saved).unwrap(),
        &state,
    );

    assert_eq!(problems.len(), 4, "{problems:?}");
    assert_eq!(commands(&plan(&restored)), Vec::<String>::new());
}

#[test]
fn a_setting_is_filed_under_its_own_variable() {
    let state = snapshot(DEFAULTS, "bgrt");
    let mut changes = ChangeSet::default();

    GrubDefaults::want(&mut changes, timeout(3), &state);
    GrubDefaults::want(&mut changes, GrubSetting::Theme(None), &Snapshot::default());
    changes.want::<GrubDefaults>(GrubKey::Default, timeout(9), &state);

    assert_eq!(
        GrubDefaults::queued(&changes, GrubKey::Timeout),
        Some(timeout(3))
    );
    assert_eq!(GrubDefaults::queued(&changes, GrubKey::Default), None);
    assert_eq!(
        changes
            .views()
            .into_iter()
            .map(|view| view.key)
            .collect::<Vec<String>>(),
        ["timeout", "theme"]
    );
}

#[test]
fn a_saved_setting_comes_back_under_its_own_variable() {
    let saved = serde_json::from_value(serde_json::json!({
        "version": crate::SAVED_VERSION,
        "changes": [{
            "domain": "grub-settings",
            "item": "timeout",
            "wish": { "key": "timeout", "value": 3_i64 },
        }],
    }))
    .unwrap();

    let mut restored = ChangeSet::default();
    let problems = domains::all()
        .unwrap()
        .restore(&mut restored, &saved, &Snapshot::default());

    assert_eq!(problems, []);
    assert_eq!(
        GrubDefaults::queued(&restored, GrubKey::Timeout),
        Some(timeout(3))
    );
    assert_eq!(restored.views()[0].key, "timeout");
    assert_eq!(restored.len(), 1);
}

#[test]
fn a_setting_saved_under_another_variable_is_refused() {
    let saved = serde_json::from_value(serde_json::json!({
        "version": crate::SAVED_VERSION,
        "changes": [{
            "domain": "grub-settings",
            "item": "theme",
            "wish": { "key": "timeout", "value": 3_i64 },
        }],
    }))
    .unwrap();

    let mut restored = ChangeSet::default();
    let problems = domains::all()
        .unwrap()
        .restore(&mut restored, &saved, &Snapshot::default());

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(restored.is_empty());
}

#[test]
fn a_cleared_value_and_the_last_booted_entry_have_sentences_of_their_own() {
    let summary = |setting: GrubSetting| GrubDefaults::summary(&setting.key(), &setting);

    assert_eq!(
        summary(GrubSetting::Default(DefaultEntry::Saved)),
        Summary::plain("grub-default-saved")
    );
    assert_eq!(
        summary(parameters("")),
        Summary::plain("grub-parameters-none")
    );
    assert_eq!(
        summary(parameters("quiet splash")),
        Summary::of("grub-parameters", "quiet splash")
    );
    assert_eq!(
        summary(GrubSetting::Background(None)),
        Summary::plain("grub-background-none")
    );
    assert_eq!(
        summary(GrubSetting::Theme(None)),
        Summary::plain("grub-theme-none")
    );
    assert_eq!(
        summary(GrubSetting::SaveDefault(true)),
        Summary::plain("grub-save-default-on")
    );
}

/// The file shows the new timeout, but the menu was never generated.
#[test]
fn a_failed_grub_mkconfig_keeps_the_setting_queued() {
    let mut changes = ChangeSet::default();
    GrubDefaults::want(&mut changes, timeout(3), &snapshot(DEFAULTS, "bgrt"));

    stopped_at(&mut changes, 2);
    changes.reconcile(&snapshot("GRUB_TIMEOUT=3\n", "bgrt"));

    assert_eq!(outcomes(&changes), [("timeout".to_owned(), failed_at(2))]);
}

/// One step sets the theme and rebuilds the initramfs: the theme shows
/// while the rebuild failed.
#[test]
fn a_failed_initramfs_rebuild_keeps_the_theme_queued() {
    let mut changes = ChangeSet::default();
    changes.want::<PlymouthThemes>(
        Single,
        plymouth_theme("spinner"),
        &snapshot(DEFAULTS, "bgrt"),
    );

    stopped_at(&mut changes, 1);
    changes.reconcile(&snapshot(DEFAULTS, "spinner"));

    assert_eq!(
        outcomes(&changes),
        [("plymouth-theme".to_owned(), failed_at(1))]
    );
}
