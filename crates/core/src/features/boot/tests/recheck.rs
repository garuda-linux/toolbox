//! A restored queue meets the page's guards once the file and the
//! Plymouth themes are known.

use toolbox_plan::{GrubKey, GrubSetting, GrubTimeout, KernelParameters};

use super::super::{recheck, recheck_grub};
use super::{ARCH, system, with_plymouth};
use crate::domains::{GrubDefaults, PlymouthThemes};
use crate::testing::plymouth_theme;
use crate::{ChangeSet, Single, Snapshot};

const QUOTED: &str = "GRUB_CMDLINE_LINUX_DEFAULT='quiet acpi_osi=\"Windows 2015\"'\n";

fn restore_line(changes: &mut ChangeSet, words: &str) {
    let list = KernelParameters::parse(words).unwrap();

    GrubDefaults::want(
        changes,
        GrubSetting::KernelParameters(list),
        &Snapshot::default(),
    );
}

#[test]
fn a_restored_plymouth_theme_that_is_not_installed_is_dropped() {
    let snapshot = with_plymouth(system(ARCH, &["plymouth"]));
    let mut changes = ChangeSet::default();

    changes.want::<PlymouthThemes>(Single, plymouth_theme("no-such-theme"), &snapshot);

    assert_eq!(recheck(&mut changes, &snapshot), Some(1));
    assert!(changes.is_empty());
}

#[test]
fn an_installed_restored_plymouth_theme_keeps_its_wish() {
    let snapshot = with_plymouth(system(ARCH, &["plymouth"]));
    let mut changes = ChangeSet::default();

    changes.want::<PlymouthThemes>(Single, plymouth_theme("spinner"), &snapshot);
    assert_eq!(recheck(&mut changes, &snapshot), Some(0));
    assert_eq!(changes.len(), 1);
}

#[test]
fn a_restored_plymouth_theme_waits_for_the_theme_list() {
    let snapshot = system(ARCH, &["plymouth"]);
    let mut changes = ChangeSet::default();

    changes.want::<PlymouthThemes>(Single, plymouth_theme("no-such-theme"), &snapshot);

    assert_eq!(recheck(&mut changes, &snapshot), None);
    assert_eq!(changes.len(), 1);
}

/// The page leaves a command line it cannot write back word for word
/// alone, so a restored one would overwrite what it never showed.
#[test]
fn a_restored_command_line_over_a_custom_one_is_dropped() {
    let mut changes = ChangeSet::default();
    let timeout = GrubSetting::Timeout(GrubTimeout::new(9).unwrap());

    restore_line(&mut changes, "quiet splash");
    GrubDefaults::want(&mut changes, timeout.clone(), &Snapshot::default());

    assert_eq!(recheck_grub(&mut changes, &system(QUOTED, &[])), Some(1));
    assert_eq!(
        GrubDefaults::queued(&changes, GrubKey::KernelParameters),
        None
    );
    assert_eq!(
        GrubDefaults::queued(&changes, GrubKey::Timeout),
        Some(timeout)
    );
}

#[test]
fn a_restored_command_line_over_a_plain_one_stays() {
    let mut changes = ChangeSet::default();

    restore_line(&mut changes, "quiet splash");

    assert_eq!(recheck_grub(&mut changes, &system(ARCH, &[])), Some(0));
    assert_eq!(changes.len(), 1);
}

#[test]
fn a_restored_grub_setting_waits_for_the_file() {
    let mut changes = ChangeSet::default();

    restore_line(&mut changes, "quiet splash");

    assert_eq!(recheck_grub(&mut changes, &Snapshot::default()), None);
    assert_eq!(changes.len(), 1);
}
