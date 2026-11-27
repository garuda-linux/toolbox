//! Every repair as the helper runs it, on each boot loader it fits.

use super::{as_run, every_repair};
use crate::procedure::Invocation;
use crate::{EspPlace, InstalledLoader, LoaderRefusal, RepairWork, RootAction};

/// Every repair as the helper runs it on each boot loader, where it fits.
/// A refusal must be the one the work earns, and the count pins the list,
/// so no test that reads these runs can pass on a list that lost them.
pub(super) fn every_run() -> Vec<(RootAction, Vec<Invocation>)> {
    let loaders = [
        InstalledLoader::Grub,
        InstalledLoader::SystemdBoot {
            esp: Some(EspPlace::Efi),
        },
    ];

    let mut runs = Vec::new();

    for action in every_repair() {
        for loader in loaders {
            match as_run(&action, loader) {
                Ok(steps) => runs.push((action.clone(), steps)),
                Err(refusal) => assert_eq!(refusal, expected_refusal(&action), "{action:?}"),
            }
        }
    }

    // Grub takes all 18 repairs, systemd-boot the 6 UEFI installs.
    assert_eq!(runs.len(), 24);
    runs
}

/// systemd-boot names no ESP for a regeneration and takes no BIOS install.
fn expected_refusal(action: &RootAction) -> LoaderRefusal {
    let RootAction::RepairBoot {
        system: _,
        work,
        passphrase: _,
    } = action
    else {
        panic!("no repair: {action:?}");
    };

    match work {
        RepairWork::RegenerateConfig { esp: _ } => LoaderRefusal::NoEspNamed,
        RepairWork::InstallBios { disk: _ } => LoaderRefusal::NotUefi,
        RepairWork::InstallUefi { esp: _, id: _ } => {
            panic!("a UEFI install fits systemd-boot: {action:?}")
        }
    }
}
