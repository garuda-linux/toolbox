//! The build order of one AUR package, as `paru -Po --aur <name>` prints
//! it for the user, without root.

#[cfg(test)]
mod tests;

use toolbox_plan::PackageName;
use toolbox_state::aur::order::{AurOrder, parse_order};

use super::Paru;
use crate::error::ProbeError;

pub trait AurOrderSource: Send + Sync {
    /// The repo packages `package` needs and the AUR packages to build for
    /// it, in order.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without paru or as root, a failed
    /// [`ProbeError`] when paru fails or prints a broken order.
    fn order(&self, package: &PackageName) -> Result<AurOrder, ProbeError>;
}

impl AurOrderSource for Paru {
    /// paru 2.2.1 exits with 0 for a whole order and with 1 when it names
    /// a missing package, both with nothing on stderr.
    fn order(&self, package: &PackageName) -> Result<AurOrder, ProbeError> {
        let finished = self.run(
            &["-Po", "--aur", "--", package.as_str()],
            "no AUR build order is read",
        )?;

        let order = |text: &str| {
            parse_order(text).map_err(|error| ProbeError::Invalid {
                what: format!("the build order of {package}"),
                error: error.into(),
            })
        };

        match finished.status.code() {
            Some(0) => order(&finished.stdout),
            Some(1) if finished.stderr.trim().is_empty() => match order(&finished.stdout)? {
                named if !named.missing.is_empty() && answers(&named, package) => Ok(named),
                _ => Err(finished.failure()),
            },
            Some(_) | None => Err(finished.failure()),
        }
    }
}

/// paru exits with 1 for other errors too. Only an order that names the
/// asked package, as a build or as missing, is its answer.
fn answers(order: &AurOrder, package: &PackageName) -> bool {
    order.builds.iter().any(|build| build.name == *package)
        || order
            .missing
            .iter()
            .any(|missing| missing.as_str() == package.as_str())
}
