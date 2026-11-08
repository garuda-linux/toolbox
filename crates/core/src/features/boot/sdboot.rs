//! On systemd-boot the parameters live in `/etc/kernel/cmdline`, and the
//! Plymouth theme shares the rebuild of the UKIs with them.

use toolbox_plan::KernelParameters;

use super::Refusal;
use crate::domains::{BootLoader, SdBootKernel, SdBootKernelDomain};
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
