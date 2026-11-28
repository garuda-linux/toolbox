//! The build order of one AUR package from `paru -Po --aur <name>`, read without root.

#[cfg(test)]
mod tests;

use toolbox_plan::PackageName;
use toolbox_state::aur::order::{AurOrder, parse_order};

use super::Paru;
use crate::error::{ProbeError, Subject};

pub trait AurOrderSource: Send + Sync {
    /// The repo packages and AUR builds `package` needs, in order.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without paru or as root, else paru's failure.
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
                what: Subject::BuildOrder {
                    package: package.to_string(),
                },
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

/// The build orders of a self-test build's dry run: the captured `paru -Po`
/// answers of two AUR builds and of a name the AUR does not have. Any other
/// name has no answer.
#[cfg(any(feature = "selftest", feature = "testing"))]
#[derive(Debug, Default)]
pub struct SampleAurOrders;

#[cfg(any(feature = "selftest", feature = "testing"))]
impl AurOrderSource for SampleAurOrders {
    fn order(&self, package: &PackageName) -> Result<AurOrder, ProbeError> {
        use toolbox_state::testing::aur::order::{
            AUR_SYNC_VOTE_STDOUT, MISSING_STDOUT, OCTOPI_GIT_STDOUT,
        };

        let text = match package.as_str() {
            "octopi-git" => OCTOPI_GIT_STDOUT,
            "aur-sync-vote" => AUR_SYNC_VOTE_STDOUT,
            "no-such-package-toolbox" | "toolbox-sample-git" => MISSING_STDOUT,
            _ => return Err(ProbeError::Unavailable),
        };

        parse_order(text).map_err(|error| ProbeError::Invalid {
            what: Subject::SampleBuildOrder {
                package: package.to_string(),
            },
            error: error.into(),
        })
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
