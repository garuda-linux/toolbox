//! The build orders of queued AUR packages, read for one run of builds.
//! Nothing calls this until the review step exists. `source` runs paru,
//! so a caller runs this inside `spawn_blocking`.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::sync::Arc;

use toolbox_core::domains::BuildOrder;
use toolbox_exec::RouteProblem;
use toolbox_plan::{BuildRun, InvalidName, PackageName};
use toolbox_system::{AurOrderSource, ProbeError};
use toolbox_text::OutsideText;

/// Why a package has no build order.
#[derive(Debug)]
pub enum NoOrder {
    /// paru could not be asked, or its answer did not read.
    Unread(ProbeError),
    /// The route to the build folder is refused, so paru was not asked.
    Route(Arc<RouteProblem>),
    /// The order lists these names as missing, so nothing can be built.
    Missing(Vec<OutsideText>),
    /// The order does not name the package as an AUR build.
    NotNamed,
}

/// One outcome per package, in the order they were asked.
pub type Orders = Vec<(PackageName, Result<BuildOrder, NoOrder>)>;

/// Reads the build order of each of `packages`. Every order of the call
/// shares one run, minted from `counter`.
///
/// # Errors
///
/// [`InvalidName`] when the run cannot be minted from `counter`. No order
/// is read then.
pub fn read_orders(
    packages: &[PackageName],
    source: &dyn AurOrderSource,
    installed: &BTreeSet<PackageName>,
    route: Result<(), RouteProblem>,
    counter: u64,
) -> Result<Orders, InvalidName> {
    let run = BuildRun::new(format!("{counter:x}"))?;
    let route = route.map_err(Arc::new);

    Ok(packages
        .iter()
        .map(|package| {
            let outcome = match &route {
                Err(problem) => Err(NoOrder::Route(Arc::clone(problem))),
                Ok(()) => order_of(package, source, installed, &run),
            };

            (package.clone(), outcome)
        })
        .collect())
}

fn order_of(
    package: &PackageName,
    source: &dyn AurOrderSource,
    installed: &BTreeSet<PackageName>,
    run: &BuildRun,
) -> Result<BuildOrder, NoOrder> {
    let order = source.order(package).map_err(NoOrder::Unread)?;

    if !order.missing.is_empty() {
        return Err(NoOrder::Missing(order.missing));
    }

    if !order.builds.iter().any(|build| build.name == *package) {
        return Err(NoOrder::NotNamed);
    }

    let named = order
        .repo
        .iter()
        .map(|install| &install.name)
        .chain(order.builds.iter().map(|build| &build.name));

    let installed = named
        .filter(|name| installed.contains(*name))
        .cloned()
        .collect();

    Ok(BuildOrder {
        order,
        run: run.clone(),
        installed,
    })
}
