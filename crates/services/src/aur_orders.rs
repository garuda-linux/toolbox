//! The build orders of queued AUR packages, read for one run of builds and
//! filled into the queue.
//!
//! `source` runs paru, so a caller runs this inside `spawn_blocking`. The
//! command line calls it before every plan, the window before every Apply.

mod reads;
#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::sync::Arc;

pub use reads::OrderReads;
use toolbox_core::domains::{BuildOrder, InstalledPackages};
use toolbox_core::features::packages::aur::{give_order, needing_orders};
use toolbox_core::{ChangeSet, Snapshot};
pub use toolbox_exec::RouteProblem;
use toolbox_plan::{BuildRun, InvalidName, PackageName};
use toolbox_state::aur::RouteKind;
use toolbox_system::{AurOrderSource, ProbeError};
use toolbox_text::OutsideText;

use crate::queue::Queue;

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

impl NoOrder {
    /// The Fluent id of the reason, whose `$name` is the package.
    pub const fn text_id(&self) -> &'static str {
        match self {
            Self::Unread(_) => "aur-order-unread",
            Self::Route(_) => "aur-order-route",
            Self::Missing(_) => "aur-order-missing",
            Self::NotNamed => "aur-order-not-named",
        }
    }
}

/// One outcome per package, in the order they were asked.
pub type Orders = Vec<(PackageName, Result<BuildOrder, NoOrder>)>;

/// A run id for one read, from the clock, so a run folder left by an
/// earlier session is never reused.
pub fn run_counter() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| {
            u64::try_from(since.as_nanos()).unwrap_or(u64::MAX)
        })
}

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

/// The build orders of every package of `packages`, `None` unless each one
/// came back. A split upgrade plans from these, and paru's upgrade takes
/// over otherwise.
pub fn read_every(
    packages: &[PackageName],
    source: &dyn AurOrderSource,
    installed: &BTreeSet<PackageName>,
    route: Result<(), RouteProblem>,
    counter: u64,
) -> Option<Vec<BuildOrder>> {
    let outcomes = read_orders(packages, source, installed, route, counter).ok()?;

    outcomes
        .into_iter()
        .map(|(_, outcome)| outcome.ok())
        .collect()
}

/// Gives each queued build of `orders` its order, in memory. A build whose
/// order did not come back waits again. Returns the reason of each package
/// that has no order. A paru build keeps its wish.
pub fn fill(queue: &mut Queue, snapshot: &Snapshot, orders: Orders) -> Vec<(PackageName, NoOrder)> {
    queue.edit(snapshot, |changes, snapshot| {
        fill_changes(changes, snapshot, orders)
    })
}

/// [`fill`] on a change set, for the command line's plan.
pub(crate) fn fill_changes(
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
    orders: Orders,
) -> Vec<(PackageName, NoOrder)> {
    let mut missing = Vec::new();

    for (package, outcome) in orders {
        match outcome {
            Ok(order) => give_order(changes, snapshot, &package, Some(order)),
            Err(reason) => {
                give_order(changes, snapshot, &package, None);
                missing.push((package, reason));
            }
        }
    }

    missing
}

/// Reads the orders of every queued build that needs one, against
/// `route`, and fills them into `changes`. Returns the reason of each
/// build without an order, empty when every queued build is built.
///
/// # Errors
///
/// [`InvalidName`] when the run cannot be minted from `counter`.
pub fn read_into(
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
    source: &dyn AurOrderSource,
    route: Result<(), RouteProblem>,
    counter: u64,
) -> Result<Vec<(PackageName, NoOrder)>, InvalidName> {
    let packages = needing(changes);
    if packages.is_empty() {
        return Ok(Vec::new());
    }

    let installed = snapshot
        .part::<InstalledPackages>()
        .map(|installed| installed.0.clone())
        .unwrap_or_default();

    let orders = read_orders(&packages, source, &installed, route, counter)?;

    Ok(fill_changes(changes, snapshot, orders))
}

/// The queued builds whose order an apply reads before its plan.
pub fn needing(changes: &ChangeSet) -> Vec<PackageName> {
    needing_orders(changes)
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

/// The Fluent id of why a setup takes paru's own way, the `$reason` of
/// `aur-old-path`.
pub const fn route_text_id(kind: RouteKind) -> &'static str {
    match kind {
        RouteKind::NoAccount => "aur-route-no-account",
        RouteKind::Linked => "aur-route-linked",
        RouteKind::Shared => "aur-route-shared",
        RouteKind::ThroughSudo => "aur-route-through-sudo",
        RouteKind::Other => "aur-route-other",
    }
}
