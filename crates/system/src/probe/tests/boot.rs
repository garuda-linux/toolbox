use std::sync::Arc;

use toolbox_state::boot::{BootLoader, CurrentGrub, KernelCmdline, PlymouthState, VirtualMachine};

use super::super::FailedProbes;
use super::fakes::{FakeBoot, FakePackages};
use super::{Probe, ProbeIssue, host, installed, probe_snapshot, sources};
use crate::BootSource;
use crate::error::ProbeError;

/// GRUB's answers, but the loader cannot be read.
struct UnreadableLoader;

impl BootSource for UnreadableLoader {
    fn grub(&self) -> Result<CurrentGrub, ProbeError> {
        FakeBoot.grub()
    }

    fn plymouth(&self) -> Result<PlymouthState, ProbeError> {
        FakeBoot.plymouth()
    }

    fn virtual_machine(&self) -> Result<VirtualMachine, ProbeError> {
        FakeBoot.virtual_machine()
    }

    fn loader(&self) -> Result<BootLoader, ProbeError> {
        Err(ProbeError::Failed(
            "install.conf: Is a directory".to_owned(),
        ))
    }

    fn kernel_cmdline(&self) -> Result<KernelCmdline, ProbeError> {
        FakeBoot.kernel_cmdline()
    }
}

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

#[test]
fn an_unreadable_loader_is_unknown_and_named() {
    let mut sources = sources(FakePackages(Ok(installed())), host());
    sources.boot = Arc::new(UnreadableLoader);

    let (snapshot, issues) = probe_snapshot(&sources);

    assert_eq!(snapshot.part::<BootLoader>(), Some(&BootLoader::Unknown));
    assert_eq!(
        issues,
        [ProbeIssue {
            probe: Probe::BootLoader,
            error: ProbeError::Failed("install.conf: Is a directory".to_owned()),
        }]
    );
}
