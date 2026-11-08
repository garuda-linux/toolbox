use toolbox_plan::{KernelParameters, PlymouthTheme};
use toolbox_state::boot::KernelCmdline;

use super::{KernelSetting, SdBootKernel, SdBootKernelDomain};
use crate::testing::commands;
use crate::{ChangeSet, Snapshot, plan};

/// `/etc/kernel/cmdline` of the Garuda Lima image.
const CMDLINE: &str = include_str!("../../../../../state/src/boot/fixtures/kernel-cmdline");

fn image() -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(SdBootKernel {
        cmdline: KernelCmdline::read(CMDLINE, false),
        theme: None,
    });
    snapshot
}

fn current() -> KernelParameters {
    KernelParameters::parse(CMDLINE).unwrap()
}

fn theme() -> PlymouthTheme {
    PlymouthTheme::new("garuda-dr460nized").unwrap()
}

#[test]
fn the_current_command_line_drops_out() {
    let state = image();
    let mut changes = ChangeSet::default();

    SdBootKernelDomain::want(&mut changes, KernelSetting::Parameters(current()), &state);

    assert!(changes.is_empty());
}

#[test]
fn a_new_command_line_is_written_and_the_ukis_built_without_dracut() {
    let state = image();
    let mut changes = ChangeSet::default();

    SdBootKernelDomain::want(
        &mut changes,
        KernelSetting::Parameters(current().without("quiet")),
        &state,
    );

    assert_eq!(
        commands(&plan(&changes)),
        [
            "write root=UUID=cd2fff21-4bb7-40f3-a158-406f545e11e6 rootflags=subvol=@ rw loglevel=3 console=tty0 console=ttyS0,115200 to /etc/kernel/cmdline",
            "kernel-install-uki add, or kernel-install add-all where kernel-install-uki is not installed",
        ]
    );
}

#[test]
fn a_theme_and_a_command_line_share_one_rebuild() {
    let state = image();
    let mut changes = ChangeSet::default();

    SdBootKernelDomain::want(&mut changes, KernelSetting::PlymouthTheme(theme()), &state);
    SdBootKernelDomain::want(
        &mut changes,
        KernelSetting::Parameters(current().without("quiet")),
        &state,
    );

    let shown = commands(&plan(&changes));

    assert_eq!(shown.len(), 4, "{shown:?}");
    assert!(shown[0].starts_with("write root=UUID="));
    assert_eq!(shown[1], "plymouth-set-default-theme garuda-dr460nized");
    assert!(shown[2].starts_with("dracut-rebuild"));
    assert!(shown[3].starts_with("kernel-install-uki add"));
}

#[test]
fn the_theme_in_use_drops_out() {
    let mut state = image();
    let mut changes = ChangeSet::default();

    state.insert(SdBootKernel {
        cmdline: KernelCmdline::read(CMDLINE, false),
        theme: Some(theme()),
    });
    SdBootKernelDomain::want(&mut changes, KernelSetting::PlymouthTheme(theme()), &state);

    assert!(changes.is_empty());
}

#[test]
fn queued_values_are_read_back_by_their_key() {
    let state = image();
    let mut changes = ChangeSet::default();

    SdBootKernelDomain::want(&mut changes, KernelSetting::PlymouthTheme(theme()), &state);

    assert_eq!(SdBootKernelDomain::queued_theme(&changes), Some(theme()));
    assert_eq!(SdBootKernelDomain::queued_parameters(&changes), None);
}
