//! Whether the helper accepts the calling user's way to the build folder.
//! The answer is one snapshot part, which the split path reads before it
//! queues a build.

use std::path::PathBuf;

use toolbox_exec::{RouteProblem, route};
use toolbox_state::aur::{AurRoute, RouteKind};

/// The route check of the helper, for the user who runs the app.
pub trait RouteSource: Send + Sync {
    /// # Errors
    ///
    /// [`RouteProblem`] for the first folder on the way the helper refuses.
    fn route(&self) -> Result<(), RouteProblem>;
}

/// The route the helper takes for `uid` below `root`, as the root helper
/// checks it.
#[derive(Clone, Debug)]
pub struct SystemRoute {
    root: PathBuf,
    uid: u32,
}

impl SystemRoute {
    pub fn system() -> Self {
        Self {
            root: PathBuf::from("/"),
            uid: rustix::process::getuid().as_raw(),
        }
    }
}

impl RouteSource for SystemRoute {
    fn route(&self) -> Result<(), RouteProblem> {
        route(&self.root, self.uid).map(|_held| ())
    }
}

/// The route of a self-test build's dry run: the helper accepts the user.
#[cfg(any(feature = "selftest", feature = "testing"))]
#[derive(Clone, Copy, Debug, Default)]
pub struct SampleRoute;

#[cfg(any(feature = "selftest", feature = "testing"))]
impl RouteSource for SampleRoute {
    fn route(&self) -> Result<(), RouteProblem> {
        Ok(())
    }
}

/// The snapshot part for the answer of `source`.
pub fn probe_route(source: &dyn RouteSource) -> AurRoute {
    match source.route() {
        Ok(()) => AurRoute::Accepted,
        Err(problem) => AurRoute::Refused {
            kind: kind_of(&problem),
            detail: problem.to_string(),
        },
    }
}

const fn kind_of(problem: &RouteProblem) -> RouteKind {
    match problem {
        RouteProblem::NoAccount(_) => RouteKind::NoAccount,
        RouteProblem::Home(_)
        | RouteProblem::Cache(_)
        | RouteProblem::Folder { name: _, error: _ } => RouteKind::Linked,
        RouteProblem::Shared(_) => RouteKind::Shared,
        RouteProblem::Passwd(_) | RouteProblem::Foreign(_) => RouteKind::Other,
    }
}

#[cfg(test)]
mod tests;
