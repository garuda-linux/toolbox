//! A click on Install or Remove, queued in the packages domain like
//! Discover's apps.

#[cfg(test)]
mod tests;

use toolbox_plan::{PackageName, Protection};

use super::protected;
use super::protected::CascadeRefusal;
use crate::domains::{InstalledPackages, Packages, SyncPackages};
use crate::features::kernels;
use crate::{ChangeSet, Snapshot, Toggle};

#[derive(Clone, Debug, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum Refusal {
    /// No configured repository has the package.
    NotOffered,
    /// The installed or the offered packages are not known yet.
    Unread,
    /// The kernel runs right now.
    RunningKernel,
    /// Which kernel runs is not known, so no kernel may go.
    RunningUnknown,
    /// The last installed kernel, without which the system does not boot.
    LastKernel,
    /// `toolbox_plan::PROTECTED` keeps the package, for this reason.
    Protected(Protection),
    /// The removal would take this protected dependency along.
    TakesAlong(Protection, PackageName),
}

impl From<CascadeRefusal> for Refusal {
    fn from(refusal: CascadeRefusal) -> Self {
        match refusal {
            CascadeRefusal::Unread => Self::Unread,
            CascadeRefusal::TakesAlong(why, along) => Self::TakesAlong(why, along),
        }
    }
}

impl From<kernels::Refusal> for Refusal {
    fn from(refusal: kernels::Refusal) -> Self {
        match refusal {
            kernels::Refusal::Running => Self::RunningKernel,
            kernels::Refusal::RunningUnknown => Self::RunningUnknown,
            kernels::Refusal::LastKernel => Self::LastKernel,
        }
    }
}

impl Refusal {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}

/// The removals queued in the packages domain plus `adding`: the targets
/// of the next apply's one `pacman -Rns`.
pub fn removal_batch(changes: &ChangeSet, adding: &[PackageName]) -> Vec<PackageName> {
    let mut batch: Vec<PackageName> = changes
        .queued::<Packages>()
        .into_iter()
        .filter(|(_, wish)| !wish.is_on())
        .map(|(package, _)| package)
        .collect();

    batch.extend(adding.iter().cloned());
    batch
}

/// The protected list, what the batch takes along and the Kernels page's
/// rules, which every removal of an installed package meets.
///
/// # Errors
///
/// [`Refusal::Protected`], [`Refusal::TakesAlong`], [`Refusal::Unread`]
/// without the dependency graph, or a kernel refusal.
pub fn may_remove(
    package: &PackageName,
    changes: &ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    protected::may_remove(package).map_err(Refusal::Protected)?;
    let batch = removal_batch(changes, std::slice::from_ref(package));
    protected::may_remove_with_cascade(&batch, snapshot)?;
    kernels::may_remove(package, changes, snapshot)?;
    Ok(())
}

/// # Errors
///
/// [`Refusal`] when the snapshot cannot tell or no repository offers a
/// package to install.
pub fn toggle(
    package: &PackageName,
    wish: Toggle,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refusal> {
    let installed = snapshot
        .part::<InstalledPackages>()
        .ok_or(Refusal::Unread)?;

    // Taking a removal back needs no repository, the package is there.
    if wish.is_on() && !installed.0.contains(package) {
        let offered = snapshot.part::<SyncPackages>().ok_or(Refusal::Unread)?;
        if !offered.0.contains(package) {
            return Err(Refusal::NotOffered);
        }
    }

    if !wish.is_on() && installed.0.contains(package) {
        may_remove(package, changes, snapshot)?;
    }

    changes.want::<Packages>(package.clone(), wish, snapshot);
    Ok(())
}
