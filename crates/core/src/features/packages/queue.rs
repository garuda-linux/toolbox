//! A click on Install or Remove, queued in the packages domain like
//! Discover's apps.

#[cfg(test)]
mod tests;

use toolbox_plan::{PackageName, Protection};

use super::protected;
use super::protected::CascadeRefusal;
use crate::domains::{InstalledPackages, Packages, SyncPackages};
use crate::features::{Refused, kernels};
use crate::{ChangeSet, Snapshot, Toggle};

/// The default is what `strum::EnumIter` puts into the `Package` variant
/// of the kernel and locale refusals.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum Refusal {
    /// No configured repository has the package.
    NotOffered,
    /// The installed or the offered packages are not known yet.
    #[default]
    Unread,
    /// The kernel runs right now.
    RunningKernel,
    /// Which kernel runs is not known, so no kernel may go.
    RunningUnknown,
    /// The last installed kernel, without which the system does not boot.
    LastKernel,
    /// `toolbox_plan::PROTECTED` keeps the package, for this reason.
    Protected(Protection),
    /// The removal would take a protected dependency along, the
    /// [`Refused::subject`].
    TakesAlong(Protection),
}

impl From<kernels::Refusal> for Refusal {
    fn from(refusal: kernels::Refusal) -> Self {
        match refusal {
            kernels::Refusal::Running => Self::RunningKernel,
            kernels::Refusal::RunningUnknown => Self::RunningUnknown,
            kernels::Refusal::LastKernel => Self::LastKernel,
            kernels::Refusal::Package(refusal) => refusal,
        }
    }
}

impl Refusal {
    pub fn of(self, package: &PackageName) -> Refused<Self, PackageName> {
        Refused::new(self, package.clone())
    }
}

/// A dependency it would take along is what the text names.
fn cascade(refusal: CascadeRefusal, package: &PackageName) -> Refused<Refusal, PackageName> {
    match refusal {
        CascadeRefusal::Unread => Refusal::Unread.of(package),
        CascadeRefusal::TakesAlong(why, along) => Refused::new(Refusal::TakesAlong(why), along),
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
) -> Result<(), Refused<Refusal, PackageName>> {
    may_remove_all(std::slice::from_ref(package), changes, snapshot)
}

/// [`may_remove`] for packages that go together. Every page and command
/// that queues a removal in the packages domain asks this first.
///
/// # Errors
///
/// As [`may_remove`], naming the first package that is refused.
pub(crate) fn may_remove_all(
    packages: &[PackageName],
    changes: &ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refused<Refusal, PackageName>> {
    protection_allows(packages, changes, snapshot)?;

    for package in packages {
        kernels::may_remove(package, changes, snapshot)
            .map_err(|refusal| Refusal::from(refusal).of(package))?;
    }

    Ok(())
}

/// The protected list and what the batch takes along: [`may_remove_all`]
/// without the Kernels page's rules, for that page, which applies them
/// to its own list.
///
/// # Errors
///
/// [`Refusal::Protected`], [`Refusal::TakesAlong`] or
/// [`Refusal::Unread`].
pub(crate) fn protection_allows(
    packages: &[PackageName],
    changes: &ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refused<Refusal, PackageName>> {
    let Some(first) = packages.first() else {
        return Ok(());
    };

    for package in packages {
        protected::may_remove(package).map_err(|why| Refusal::Protected(why).of(package))?;
    }

    let batch = removal_batch(changes, packages);
    protected::may_remove_with_cascade(&batch, snapshot).map_err(|refusal| cascade(refusal, first))
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
) -> Result<(), Refused<Refusal, PackageName>> {
    let installed = snapshot
        .part::<InstalledPackages>()
        .ok_or_else(|| Refusal::Unread.of(package))?;

    // Taking a removal back needs no repository, the package is there.
    if wish.is_on() && !installed.0.contains(package) {
        let offered = snapshot
            .part::<SyncPackages>()
            .ok_or_else(|| Refusal::Unread.of(package))?;

        if !offered.0.contains(package) {
            return Err(Refusal::NotOffered.of(package));
        }
    }

    if !wish.is_on() && installed.0.contains(package) {
        may_remove(package, changes, snapshot)?;
    }

    changes.want::<Packages>(package.clone(), wish, snapshot);
    Ok(())
}
