//! The orphans of the local database, as `pacman -Qdt` lists them.

use alpm::PackageReason;
use toolbox_plan::PackageName;

use super::{Libalpm, open_local};
use crate::error::ProbeError;
use crate::names;

/// Answers which installed packages nothing needs any more.
pub trait OrphanSource: Send + Sync {
    /// Packages installed as a dependency that no installed package
    /// requires or optionally requires, sorted by name.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without a package manager,
    /// [`ProbeError::Failed`] when the local database cannot be read.
    fn orphans(&self) -> Result<Vec<PackageName>, ProbeError>;
}

impl OrphanSource for Libalpm {
    fn orphans(&self) -> Result<Vec<PackageName>, ProbeError> {
        let handle = open_local(&self.conf()?)?;
        let local = handle.localdb().pkgs();
        let orphans = local.iter().filter(|package| {
            package.reason() == PackageReason::Depend
                && package.required_by().is_empty()
                && package.optional_for().is_empty()
        });

        Ok(names::valid(orphans.map(|package| package.name()))
            .into_iter()
            .collect())
    }
}
