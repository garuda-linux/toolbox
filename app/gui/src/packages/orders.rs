//! The window's reads of build orders for the queue: one read off the frame,
//! landed into the queue by App, and the Apply that waits for a fresh one.

use std::sync::Arc;

use toolbox_core::domains::InstalledPackages;
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::PackageName;
use toolbox_services::aur_orders::{NoOrder, Orders, read_orders, run_counter};

use super::{Landed, Packages};

/// The outcome of one read of build orders, as the window applies it to
/// the queue.
#[derive(Debug)]
pub enum Landing {
    Read(Orders),
    /// The read could not run, so no order came back.
    Failed,
}

impl Packages {
    /// Asks for the build orders of the queue, on a blocking thread. A
    /// read starts for a waiting build no read named yet, or for every
    /// queued build when `force` is set, as an apply asks. Returns whether
    /// one started. One read runs at a time, so a build queued meanwhile
    /// is named by the next ask, once this one landed.
    pub fn ask_orders(&mut self, changes: &ChangeSet, snapshot: &Snapshot, force: bool) -> bool {
        let Some(jobs) = &self.jobs else {
            return false;
        };

        let Some(packages) = self.orders.ask(changes, force) else {
            return false;
        };

        let installed = snapshot
            .part::<InstalledPackages>()
            .map(|installed| installed.0.clone())
            .unwrap_or_default();

        let source = Arc::clone(&jobs.setup.orders);
        let route = Arc::clone(&jobs.setup.route);
        jobs.lane.blocking(
            "orders",
            move || {
                let route = route.route();
                read_orders(&packages, source.as_ref(), &installed, route, run_counter())
            },
            |read| {
                let landing = match read {
                    Ok(Ok(orders)) => Landing::Read(orders),
                    Ok(Err(error)) => {
                        tracing::warn!("the build orders were not read: {error}");
                        Landing::Failed
                    }
                    Err(error) => {
                        tracing::warn!("the build orders were not read: {error}");
                        Landing::Failed
                    }
                };

                Landed::Orders(landing)
            },
        );

        true
    }

    /// Whether a read of build orders runs.
    pub const fn reading_orders(&self) -> bool {
        self.orders.running()
    }

    /// Whether the running read of build orders is an apply's.
    pub const fn orders_forced(&self) -> bool {
        self.orders.forced()
    }

    /// The landed read of build orders, once, for App to apply to the queue.
    pub const fn take_orders(&mut self) -> Option<Landing> {
        self.landing.take()
    }

    /// Applies a landed read to `changes`: the orders go to the queued builds
    /// that still wait for one.
    pub fn land_orders(&mut self, changes: &mut ChangeSet, snapshot: &Snapshot, landing: Landing) {
        match landing {
            Landing::Read(orders) => self.orders.landed(changes, snapshot, orders),
            Landing::Failed => self.orders.failed(),
        }
    }

    /// Why each queued build waits for its order, from the last read.
    pub fn order_missing(&self) -> &[(PackageName, NoOrder)] {
        self.orders.missing()
    }

    /// An Apply waits for a fresh read of every order, which starts with
    /// [`Packages::ask_orders`]. `revision` is the queue's when Apply was
    /// clicked, so a queue edit meanwhile drops the Apply.
    pub const fn start_apply(&mut self, revision: u64) {
        self.apply = Some(revision);
    }

    /// The queue's revision when the waiting Apply was clicked.
    pub const fn apply_revision(&self) -> Option<u64> {
        self.apply
    }

    /// The queue changed by the window's own fill, which is no edit of the
    /// user, so the waiting Apply keeps its place.
    pub const fn rebase_apply(&mut self, revision: u64) {
        self.apply = Some(revision);
    }

    pub const fn end_apply(&mut self) {
        self.apply = None;
    }
}
