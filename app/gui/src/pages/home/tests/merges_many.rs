//! Three copies at once: pacman's `.pacnew`, the `.pacsave` of
//! `locale.gen` that the Lima VM's `pacdiff -o` listed, and GRUB's `.pacnew`.

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::features::merges::choice;
use toolbox_plan::testing::grub::LIMA_DEFAULT_GRUB;
use toolbox_plan::testing::locales::LOCALE_GEN;
use toolbox_plan::{ConfigText, MergeCopy, Resolution};
use toolbox_system::MergePair;
use toolbox_system::testing::packages::PACMAN_CONF;

use super::Shown;
use super::merges::opened;
use crate::command::{Command, HomeCommand};

const PACMAN: &str = "/etc/pacman.conf.pacnew";
const LOCALE: &str = "/etc/locale.gen.pacsave";
const GRUB: &str = "/etc/default/grub.pacnew";

/// The `.pacnew` is the capture. The file in use ignores `linux-zen`,
/// has `Color` commented out and downloads 10 packages at once.
fn pacman_current() -> String {
    PACMAN_CONF
        .replacen("\n#IgnorePkg   =\n", "\nIgnorePkg   = linux-zen\n", 1)
        .replacen("\nColor\n", "\n#Color\n", 1)
        .replacen("\nParallelDownloads = 5\n", "\nParallelDownloads = 10\n", 1)
}

/// The file in use with only `Color` taken from the `.pacnew`.
fn pacman_merged() -> String {
    PACMAN_CONF
        .replacen("\n#IgnorePkg   =\n", "\nIgnorePkg   = linux-zen\n", 1)
        .replacen("\nParallelDownloads = 5\n", "\nParallelDownloads = 10\n", 1)
}

/// The saved copy is the capture with `de_DE` added by its user.
fn locale_saved() -> String {
    format!("{LOCALE_GEN}de_DE.UTF-8 UTF-8\n")
}

/// The `.pacnew` is the capture. The file in use waits 2 seconds and boots
/// with a splash.
fn grub_current() -> String {
    LIMA_DEFAULT_GRUB
        .replacen("GRUB_TIMEOUT=5\n", "GRUB_TIMEOUT=2\n", 1)
        .replacen("loglevel=3 quiet\"", "loglevel=3 quiet splash\"", 1)
}

fn pair(copy: &str, copied: &str, current: &str) -> MergePair {
    MergePair {
        copy: MergeCopy::new(copy).unwrap(),
        copied: Ok(ConfigText::new(copied).unwrap()),
        current: Ok(Some(ConfigText::new(current).unwrap())),
    }
}

fn three() -> Shown {
    let harness = opened(vec![
        pair(PACMAN, PACMAN_CONF, &pacman_current()),
        pair(LOCALE, &locale_saved(), LOCALE_GEN),
        pair(GRUB, LIMA_DEFAULT_GRUB, &grub_current()),
    ]);

    harness.state().fixture.sent();
    harness
}

fn merge(copy: &str, resolution: Resolution, current: &str, copied: &str) -> Command {
    Command::from(HomeCommand::Merge {
        copy: MergeCopy::new(copy).unwrap(),
        choice: Some(choice(resolution, Some(current), copied)),
    })
}

fn pacman_merge() -> Command {
    merge(
        PACMAN,
        Resolution::Merged {
            text: ConfigText::new(pacman_merged()).unwrap(),
        },
        &pacman_current(),
        PACMAN_CONF,
    )
}

fn show(harness: &mut Shown, file: &str) {
    harness.get_by_role_and_label(Role::Tab, file).click();
    harness.run();
}

fn click(harness: &mut Shown, role: Role, label: &str) {
    harness.get_by_role_and_label(role, label).click();
    harness.run();
}

/// Takes the second of pacman's three changes and keeps the others.
fn take_color(harness: &mut Shown) {
    second_take(harness).scroll_to_me();
    harness.run();
    second_take(harness).click();
    harness.run();
}

fn count(harness: &Shown, label: &str) -> usize {
    harness
        .query_all_by_role_and_label(Role::RadioButton, label)
        .count()
}

fn second_take(harness: &Shown) -> egui_kittest::Node<'_> {
    harness
        .query_all_by_role_and_label(Role::RadioButton, "Take new")
        .nth(1)
        .unwrap()
}

#[test]
fn every_copy_is_listed_with_its_own_changes() {
    let mut harness = three();

    for file in ["/etc/pacman.conf", "/etc/locale.gen", "/etc/default/grub"] {
        assert!(harness.query_by_role_and_label(Role::Tab, file).is_some());
    }

    assert_eq!(count(&harness, "Take new"), 3);

    show(&mut harness, "/etc/locale.gen");
    assert_eq!(count(&harness, "Take saved"), 1);
    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Restore saved file")
            .is_some()
    );

    show(&mut harness, "/etc/default/grub");
    assert_eq!(count(&harness, "Take new"), 2);
}

#[test]
fn mixed_picks_queue_the_merged_text() {
    let mut harness = three();

    take_color(&mut harness);
    click(&mut harness, Role::Button, "Queue the merge");

    assert_eq!(harness.state().fixture.sent(), [pacman_merge()]);
}

#[test]
fn picks_stay_while_another_copy_is_shown() {
    let mut harness = three();

    take_color(&mut harness);
    show(&mut harness, "/etc/locale.gen");
    click(&mut harness, Role::RadioButton, "Take saved");
    show(&mut harness, "/etc/pacman.conf");
    click(&mut harness, Role::Button, "Queue the merge");
    show(&mut harness, "/etc/locale.gen");
    click(&mut harness, Role::Button, "Queue the merge");

    assert_eq!(
        harness.state().fixture.sent(),
        [
            pacman_merge(),
            merge(LOCALE, Resolution::TakeCopy, LOCALE_GEN, &locale_saved()),
        ]
    );
}

#[test]
fn each_copy_queues_its_own_choice() {
    let mut harness = three();

    take_color(&mut harness);
    click(&mut harness, Role::Button, "Queue the merge");
    show(&mut harness, "/etc/locale.gen");
    click(&mut harness, Role::Button, "Keep current file");
    show(&mut harness, "/etc/default/grub");
    click(&mut harness, Role::Button, "Use new file");

    let grub = grub_current();
    let saved = locale_saved();
    assert_eq!(
        harness.state().fixture.sent(),
        [
            pacman_merge(),
            merge(LOCALE, Resolution::KeepCurrent, LOCALE_GEN, &saved),
            merge(GRUB, Resolution::TakeCopy, &grub, LIMA_DEFAULT_GRUB),
        ]
    );
}
