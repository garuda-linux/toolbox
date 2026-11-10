use toolbox_plan::{KernelParameter, KernelParameters};
use toolbox_state::boot::KernelCmdline;

use super::super::{
    Flag, Refusal, add_parameter, is_protected, parameters, plymouth_queued, remove_parameter,
    set_flag, set_plymouth_theme,
};
use crate::domains::{
    BootLoader, GrubDefaults, InstalledPackages, PlymouthState, PlymouthThemes, SdBootKernel,
    SdBootKernelDomain,
};
use crate::test_support::{packages, plymouth_theme};
use crate::{ChangeSet, Single, Snapshot, Toggle};

/// `/etc/kernel/cmdline` of the Garuda Lima image and `/proc/cmdline` of
/// the Garuda live ISO.
pub(in crate::features::boot) const CMDLINE: &str = toolbox_state::testing::boot::KERNEL_CMDLINE;
pub(in crate::features::boot) const LIVE: &str = toolbox_plan::testing::host::LIVE_PROC_CMDLINE;

pub(in crate::features::boot) fn image(line: &str, installed: &[&str]) -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(BootLoader::SystemdBoot);
    snapshot.insert(SdBootKernel {
        cmdline: KernelCmdline::read(line, false),
        theme: Some(plymouth_theme("bgrt")),
    });
    snapshot.insert(PlymouthState {
        themes: vec![plymouth_theme("bgrt"), plymouth_theme("spinner")],
        current: Some(plymouth_theme("bgrt")),
    });
    snapshot.insert(InstalledPackages(packages(installed)));
    snapshot
}

fn queued_line(changes: &ChangeSet) -> Option<String> {
    SdBootKernelDomain::queued_parameters(changes).map(|list| list.to_string())
}

#[test]
fn the_parameters_come_from_the_kernel_cmdline() {
    assert_eq!(
        parameters(&image(CMDLINE, &[]), &ChangeSet::default()),
        Ok(KernelParameters::parse(CMDLINE).unwrap())
    );
}

#[test]
fn a_new_parameter_goes_to_the_kernel_cmdline_and_not_to_grub() {
    let snapshot = image(CMDLINE, &[]);
    let mut changes = ChangeSet::default();

    add_parameter(
        KernelParameter::new("nowatchdog").unwrap(),
        &mut changes,
        &snapshot,
    )
    .unwrap();

    assert!(
        queued_line(&changes)
            .unwrap()
            .ends_with(" console=ttyS0,115200 nowatchdog")
    );
    assert_eq!(changes.queued::<GrubDefaults>(), Vec::new());
}

#[test]
fn the_root_parameters_cannot_be_removed() {
    let snapshot = image(CMDLINE, &[]);
    let mut changes = ChangeSet::default();

    for word in [
        "root=UUID=cd2fff21-4bb7-40f3-a158-406f545e11e6",
        "rootflags=subvol=@",
        "rw",
    ] {
        assert!(is_protected(word), "{word}");
        assert_eq!(
            remove_parameter(word, &mut changes, &snapshot),
            Err(Refusal::ProtectedParameter),
            "{word}"
        );
    }

    assert!(changes.is_empty());
    remove_parameter("loglevel=3", &mut changes, &snapshot).unwrap();
    assert!(!queued_line(&changes).unwrap().contains("loglevel"));
}

#[test]
fn a_switch_changes_the_kernel_cmdline() {
    let snapshot = image(CMDLINE, &[]);
    let mut changes = ChangeSet::default();

    set_flag(Flag::Quiet, Toggle::Off, &mut changes, &snapshot).unwrap();

    assert!(!queued_line(&changes).unwrap().contains("quiet"));
}

/// No Garuda edition installs Plymouth on systemd-boot, so the splash
/// queues it as on GRUB.
#[test]
fn the_splash_queues_plymouth_where_it_is_missing() {
    let snapshot = image(CMDLINE, &[]);
    let mut changes = ChangeSet::default();

    set_flag(Flag::Splash, Toggle::On, &mut changes, &snapshot).unwrap();

    assert!(plymouth_queued(&changes));
    assert!(queued_line(&changes).unwrap().ends_with(" splash"));
}

#[test]
fn a_line_the_page_cannot_write_back_is_left_alone() {
    let snapshot = image(LIVE, &[]);
    let mut changes = ChangeSet::default();

    assert_eq!(
        add_parameter(
            KernelParameter::new("nowatchdog").unwrap(),
            &mut changes,
            &snapshot
        ),
        Err(Refusal::CustomKernelCmdline)
    );
    assert!(changes.is_empty());
}

#[test]
fn without_a_command_line_nothing_queues() {
    let mut snapshot = Snapshot::default();
    let mut changes = ChangeSet::default();

    snapshot.insert(BootLoader::SystemdBoot);

    assert_eq!(
        remove_parameter("quiet", &mut changes, &snapshot),
        Err(Refusal::NoKernelCmdline)
    );
}

#[test]
fn a_theme_shares_the_rebuild_of_the_kernel_cmdline() {
    let snapshot = image(CMDLINE, &["plymouth"]);
    let mut changes = ChangeSet::default();

    set_plymouth_theme(plymouth_theme("spinner"), &mut changes, &snapshot).unwrap();

    assert_eq!(
        SdBootKernelDomain::queued_theme(&changes),
        Some(plymouth_theme("spinner"))
    );
    assert_eq!(changes.wish::<PlymouthThemes>(&Single), None);
}

/// A queue from disk was never checked against this system.
#[test]
fn a_restored_theme_plymouth_lacks_is_dropped() {
    let snapshot = image(CMDLINE, &["plymouth"]);
    let mut changes = ChangeSet::default();

    SdBootKernelDomain::want(
        &mut changes,
        crate::domains::KernelSetting::PlymouthTheme(plymouth_theme("garuda-dr460nized")),
        &snapshot,
    );

    assert_eq!(super::super::recheck(&mut changes, &snapshot), Some(1));
    assert!(changes.is_empty());
}

#[test]
fn a_restored_command_line_is_dropped_where_the_line_is_left_alone() {
    let snapshot = image(LIVE, &[]);
    let mut changes = ChangeSet::default();

    SdBootKernelDomain::want(
        &mut changes,
        crate::domains::KernelSetting::Parameters(KernelParameters::parse("quiet").unwrap()),
        &snapshot,
    );

    assert_eq!(super::super::recheck_grub(&mut changes, &snapshot), Some(1));
    assert!(changes.is_empty());
}
