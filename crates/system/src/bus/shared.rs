//! One bus connection per unit scope for the unit lists and their
//! watcher, which read again on every change. A connection per list
//! would add a client to the bus each time.
//!
//! The connection is to the bus, so it outlives a restarted systemd. A
//! failed call drops it all the same: the bus itself may have gone, and
//! the next call connects again.

#[cfg(test)]
mod tests;

use parking_lot::Mutex;
use toolbox_plan::UnitScope;
use zbus::Connection;

use super::{CallError, connect};

static SHARED: Buses<Connection> = Buses::new();

/// The connections, `None` until first used or after a failure.
#[derive(Debug)]
struct Buses<C> {
    system: Mutex<Option<C>>,
    user: Mutex<Option<C>>,
}

impl<C: Clone> Buses<C> {
    pub const fn new() -> Self {
        Self {
            system: Mutex::new(None),
            user: Mutex::new(None),
        }
    }

    /// The kept connection of `scope`, or a new one from `connect`. Two
    /// callers at once may both connect, the later one is kept.
    ///
    /// # Errors
    ///
    /// What `connect` failed with.
    pub async fn get<F, R>(&self, scope: UnitScope, connect: F) -> Result<C, CallError>
    where
        F: FnOnce() -> R,
        R: Future<Output = Result<C, CallError>>,
    {
        // The lock is let go before the await.
        let kept = self.slot(scope).lock().clone();
        if let Some(connection) = kept {
            return Ok(connection);
        }

        let connection = connect().await?;
        *self.slot(scope).lock() = Some(connection.clone());
        Ok(connection)
    }

    fn forget(&self, scope: UnitScope) {
        self.slot(scope).lock().take();
    }

    const fn slot(&self, scope: UnitScope) -> &Mutex<Option<C>> {
        match scope {
            UnitScope::System => &self.system,
            UnitScope::User => &self.user,
        }
    }
}

impl<C: Clone> Default for Buses<C> {
    fn default() -> Self {
        Self::new()
    }
}

/// The one connection of `scope` the unit lists and their watcher share.
///
/// # Errors
///
/// The failure to reach the bus, classified.
pub async fn shared(scope: UnitScope) -> Result<Connection, CallError> {
    SHARED.get(scope, || connect(scope)).await
}

/// After a failed call on the [`shared`] connection, so the next one
/// connects again.
pub fn forget_shared(scope: UnitScope) {
    SHARED.forget(scope);
}
