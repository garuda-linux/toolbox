use toolbox_state::boot::{CurrentGrub, PlymouthState, VirtualMachine};

use super::super::FailedProbes;
use super::fakes::FakePackages;
use super::{Probe, host, installed, probe_snapshot, sources};
use crate::error::ProbeError;

#[test]
fn the_boot_parts_are_in_the_snapshot() {
    let (snapshot, issues) = probe_snapshot(&sources(FakePackages(Ok(installed())), host()));

    assert_eq!(issues, []);
    assert_eq!(
        snapshot.part::<CurrentGrub>().map(|grub| grub.0.timeout),
        Some(Some(7))
    );
    assert_eq!(
        snapshot
            .part::<PlymouthState>()
            .map(|plymouth| plymouth.themes.len()),
        Some(1)
    );
    assert_eq!(
        snapshot.part::<VirtualMachine>(),
        Some(&VirtualMachine(false))
    );
}

/// A page whose part is missing tells a failed read from a system that
/// lacks the thing.
#[test]
fn only_failed_probes_are_named_in_the_snapshot() {
    let (failed, _) = probe_snapshot(&sources(
        FakePackages(Err(ProbeError::Failed("locked".to_owned()))),
        host(),
    ));

    let (lacking, _) = probe_snapshot(&sources(FakePackages(Err(ProbeError::Unavailable)), host()));
    let (whole, _) = probe_snapshot(&sources(FakePackages(Ok(installed())), host()));

    assert!(
        failed
            .part::<FailedProbes>()
            .is_some_and(|probes| probes.contains(Probe::Packages))
    );
    assert!(
        lacking
            .part::<FailedProbes>()
            .is_some_and(|probes| !probes.contains(Probe::Packages))
    );
    assert_eq!(whole.part::<FailedProbes>(), Some(&FailedProbes::default()));
}
