//! On systemd-boot the parameters live in `/etc/kernel/cmdline`, and the
//! Plymouth theme shares the rebuild of the UKIs with them.

use toolbox_plan::sdboot::{LoaderEntry, LoaderKey, LoaderSetting, LoaderSettings, SdBootMenu};
use toolbox_plan::{KernelParameters, LoaderDefault, PackageName};
use toolbox_state::kernels::KernelReleases;

use super::Refusal;
use crate::domains::{BootLoader, LoaderSettingsDomain, SdBootKernel, SdBootKernelDomain};
use crate::{ChangeSet, Snapshot};

/// Without them the kernel finds no root file system to start.
pub fn is_protected(parameter: &str) -> bool {
    parameter == "rw" || parameter.starts_with("root=") || parameter.starts_with("rootflags=")
}

pub(super) fn is_sdboot(snapshot: &Snapshot) -> bool {
    snapshot.part::<BootLoader>() == Some(&BootLoader::SystemdBoot)
}

/// `None` on GRUB and while the probe has not read the command line.
pub fn kernel(snapshot: &Snapshot) -> Option<&SdBootKernel> {
    snapshot
        .part::<SdBootKernel>()
        .filter(|_| is_sdboot(snapshot))
}

pub(super) fn parameters(
    snapshot: &Snapshot,
    changes: &ChangeSet,
) -> Result<KernelParameters, Refusal> {
    let kernel = kernel(snapshot).ok_or(Refusal::NoKernelCmdline)?;

    SdBootKernelDomain::queued_parameters(changes)
        .or_else(|| kernel.cmdline.parameters.clone())
        .ok_or(Refusal::CustomKernelCmdline)
}

/// `None` for a kernel that is not installed or has no image yet.
pub fn kernel_default(
    menu: &SdBootMenu,
    kernel: &PackageName,
    snapshot: &Snapshot,
) -> Option<LoaderDefault> {
    let release = snapshot.part::<KernelReleases>()?.0.get(kernel)?;

    menu.kernel_entry(release).map(LoaderEntry::default_pattern)
}

/// The loader settings as the page shows them: the file's, each one a
/// queued change overrides.
pub fn wanted_loader(file: &LoaderSettings, changes: &ChangeSet) -> LoaderSettings {
    let mut settings = file.clone();

    for key in [
        LoaderKey::Timeout,
        LoaderKey::Default,
        LoaderKey::Editor,
        LoaderKey::AutoEntries,
        LoaderKey::AutoFirmware,
    ] {
        match LoaderSettingsDomain::queued(changes, key) {
            Some(LoaderSetting::Timeout(timeout)) => settings.timeout = timeout,
            Some(LoaderSetting::Default(default)) => settings.default = default,
            Some(LoaderSetting::Editor(on)) => settings.editor = on,
            Some(LoaderSetting::AutoEntries(on)) => settings.auto_entries = on,
            Some(LoaderSetting::AutoFirmware(on)) => settings.auto_firmware = on,
            None => {}
        }
    }

    settings
}

/// Queues a setting of systemd-boot's `loader.conf`, for the window and
/// the command line alike.
pub fn set_loader(setting: LoaderSetting, changes: &mut ChangeSet, snapshot: &Snapshot) {
    LoaderSettingsDomain::want(changes, setting, snapshot);
}
