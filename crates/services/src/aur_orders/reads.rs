//! When the window reads the build orders: one read at a time, started
//! for a queued build that no read named yet, and again for every apply.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use toolbox_core::domains::waiting_orders;
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::PackageName;

use super::{NoOrder, Orders, fill_changes, needing};

/// The read state of the build orders. A read names every queued build
/// that needs an order, so the builds of one read share its run.
#[derive(Debug, Default)]
pub struct OrderReads {
    running: bool,
    /// The running read was asked for every queued build, as an apply asks.
    forced: bool,
    /// The builds a read already named. A build without an order stays
    /// waiting and is not asked again until an apply or Read again asks.
    named: BTreeSet<PackageName>,
    missing: Vec<(PackageName, NoOrder)>,
}

impl OrderReads {
    /// The builds the next read names, when one should start now. `force`
    /// asks for every queued build, as an apply does. Without it a read
    /// starts only for a waiting build no read named yet.
    pub fn ask(&mut self, changes: &ChangeSet, force: bool) -> Option<Vec<PackageName>> {
        let queued = needing(changes);
        let waiting = waiting_orders(changes);

        self.named.retain(|name| queued.contains(name));
        self.missing.retain(|(name, _)| waiting.contains(name));

        if self.running {
            return None;
        }

        let wanted = if force {
            !queued.is_empty()
        } else {
            waiting.iter().any(|name| !self.named.contains(name))
        };

        if !wanted {
            return None;
        }

        self.running = true;
        self.forced = force;
        self.named.extend(queued.iter().cloned());
        Some(queued)
    }

    /// The read of the last [`Self::ask`] came back with `orders`. A build
    /// that left the queue meanwhile is not put back.
    pub fn landed(&mut self, changes: &mut ChangeSet, snapshot: &Snapshot, orders: Orders) {
        self.running = false;
        let missing = fill_changes(changes, snapshot, orders);

        let waiting = waiting_orders(changes);
        self.missing = missing
            .into_iter()
            .filter(|(name, _)| waiting.contains(name))
            .collect();
    }

    /// The last read could not run at all. No order came back, and no
    /// build is asked again on its own.
    pub const fn failed(&mut self) {
        self.running = false;
    }

    pub const fn running(&self) -> bool {
        self.running
    }

    /// Whether the running read was asked for every queued build.
    pub const fn forced(&self) -> bool {
        self.forced
    }

    /// Why each waiting build has no order, from the last read.
    pub fn missing(&self) -> &[(PackageName, NoOrder)] {
        &self.missing
    }
}
