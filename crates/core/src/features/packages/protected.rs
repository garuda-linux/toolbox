//! The packages `protected.toml` keeps: no page and no command removes
//! them. Every removal of a package asks here first.

#[cfg(test)]
mod tests;

use toolbox_plan::PackageName;
use toolbox_state::packages::{ProtectedPackages, Protection};

use crate::Snapshot;

/// # Errors
///
/// The reason the protected catalog gives for keeping `package`.
pub fn may_remove(package: &PackageName, snapshot: &Snapshot) -> Result<(), Protection> {
    snapshot
        .part::<ProtectedPackages>()
        .and_then(|kept| kept.0.get(package))
        .map_or(Ok(()), |why| Err(*why))
}
