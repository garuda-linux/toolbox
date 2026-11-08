//! The guards for a queue that came from disk. It was restored before
//! the system was known, so nothing checked it.

use toolbox_plan::GrubKey;

use super::{Refusal, grub, kernel, plymouth_has};
use crate::domains::{GrubDefaults, KernelKey, PlymouthState, PlymouthThemes, SdBootKernelDomain};
use crate::{ChangeSet, Single, Snapshot};

/// Drops every queued theme Plymouth does not have installed.
///
/// Returns how many went, `None` without the list of installed themes:
/// nothing can be checked then and everything stays.
pub fn recheck(changes: &mut ChangeSet, snapshot: &Snapshot) -> Option<usize> {
    snapshot.part::<PlymouthState>()?;

    let mut dropped = 0;

    for (_, theme) in changes.queued::<PlymouthThemes>() {
        if plymouth_has(&theme, snapshot) {
            continue;
        }

        changes.remove::<PlymouthThemes>(&Single);
        tracing::warn!("dropped the restored theme {theme}: Plymouth does not have it");
        dropped += 1;
    }

    if let Some(theme) = SdBootKernelDomain::queued_theme(changes)
        && !plymouth_has(&theme, snapshot)
    {
        changes.remove::<SdBootKernelDomain>(&KernelKey::PlymouthTheme);
        tracing::warn!("dropped the restored theme {theme}: Plymouth does not have it");
        dropped += 1;
    }

    Some(dropped)
}

/// Drops a queued command line when the file holds one the page leaves
/// alone ([`Refusal::CustomCommandLine`]), on GRUB and on systemd-boot.
///
/// Returns how many went, `None` without the file.
pub fn recheck_grub(changes: &mut ChangeSet, snapshot: &Snapshot) -> Option<usize> {
    if let Some(kernel) = kernel(snapshot) {
        let queued = SdBootKernelDomain::queued_parameters(changes);

        if kernel.cmdline.parameters.is_some() || queued.is_none() {
            return Some(0);
        }

        changes.remove::<SdBootKernelDomain>(&KernelKey::Parameters);
        tracing::warn!(
            "dropped the restored kernel command line: {}",
            Refusal::CustomKernelCmdline.as_str()
        );
        return Some(1);
    }

    let state = grub(snapshot).ok()?;
    let queued = GrubDefaults::queued(changes, GrubKey::KernelParameters);

    if state.parameters.is_some() || queued.is_none() {
        return Some(0);
    }

    changes.remove::<GrubDefaults>(&GrubKey::KernelParameters);
    tracing::warn!(
        "dropped the restored kernel command line: {}",
        Refusal::CustomCommandLine.as_str()
    );
    Some(1)
}
