//! The split path of `packages install --aur` on the command line: the queue
//! filled with its build orders before the plan, and the note of a setup that
//! takes paru's own way.

use toolbox_aur::AurSource;
use toolbox_core::{ChangeSet, Snapshot, plan};
use toolbox_plan::{PackageName, Plan};
use toolbox_services::aur_orders::{NoOrder, RouteProblem, read_into};
use toolbox_state::aur::{AurRoute, RouteKind};
use toolbox_system::AurOrderSource;
use toolbox_text::OutsideText;

use super::install_onto;
use crate::change::Refused;
use crate::out::{Settings, warn};
use crate::root;

/// The plan of `packages install --aur`: the builds queued as the route
/// allows, their build orders read and filled, then the plan.
///
/// # Errors
///
/// As [`install_onto`], and [`Refused::Unavailable`] for the first build
/// that has no build order. Nothing runs then.
pub fn plan_install(
    packages: &[PackageName],
    aur: &dyn AurSource,
    snapshot: &Snapshot,
    orders: &dyn AurOrderSource,
    route: Result<(), RouteProblem>,
) -> Result<Plan, Refused> {
    let mut changes = ChangeSet::default();
    install_onto(packages, aur, snapshot, &mut changes)?;

    filled(&mut changes, snapshot, orders, route, root::run_counter())?;
    Ok(plan(&changes))
}

/// Reads and fills the build orders of the queued AUR builds, the step every
/// command line plan takes before its plan.
///
/// # Errors
///
/// [`Refused::Unavailable`] naming the first build without its order.
pub fn filled(
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
    orders: &dyn AurOrderSource,
    route: Result<(), RouteProblem>,
    counter: u64,
) -> Result<(), Refused> {
    let missing = read_into(changes, snapshot, orders, route, counter)
        .map_err(|error| Refused::Unavailable(format!("no build run could be named: {error}")))?;

    match missing.first() {
        Some((package, reason)) => Err(no_order(package, reason)),
        None => Ok(()),
    }
}

/// Why `package` has no build order, with the words of the command line.
fn no_order(package: &PackageName, reason: &NoOrder) -> Refused {
    let (why, hint) = match reason {
        NoOrder::Unread(_) => (
            "paru could not be asked".to_owned(),
            "check that paru runs, then try again".to_owned(),
        ),
        NoOrder::Route(problem) => (
            "its build folder cannot be used".to_owned(),
            problem.to_string(),
        ),
        NoOrder::Missing(names) => (
            format!(
                "the AUR does not have {}",
                names
                    .iter()
                    .map(OutsideText::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            "see garuda-toolbox packages search --aur".to_owned(),
        ),
        NoOrder::NotNamed => (
            "paru does not build it from the AUR".to_owned(),
            "see garuda-toolbox packages show --aur".to_owned(),
        ),
    };

    Refused::Unavailable(format!("`{package}` has no build order: {why}\n{hint}"))
}

/// The old paru path and the reason the helper refuses the route, or `None`
/// when it accepts the route.
pub fn old_path_note(route: &AurRoute) -> Option<String> {
    match route {
        AurRoute::Accepted => None,
        AurRoute::Refused { kind, detail: _ } => Some(format!(
            "this setup uses the old paru path: {}",
            route_reason(*kind)
        )),
    }
}

const fn route_reason(kind: RouteKind) -> &'static str {
    match kind {
        RouteKind::NoAccount => "the account has no entry in /etc/passwd",
        RouteKind::Linked => {
            "the home folder or ~/.cache is a link, or a folder the build needs is missing"
        }
        RouteKind::Shared => "a folder the build uses can be written by other users",
        RouteKind::ThroughSudo => {
            "the command line asks for the password through sudo, which does not say who asked"
        }
        RouteKind::Other => "the build folder cannot be checked",
    }
}

/// Warns that the setup takes paru's own way, when the route is refused.
pub fn warn_old_path(settings: Settings, snapshot: &Snapshot) {
    if let Some(note) = snapshot.part::<AurRoute>().and_then(old_path_note) {
        warn(settings, &note);
    }
}
