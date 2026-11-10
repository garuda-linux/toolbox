//! The unit lists of the Services page, from the [`UnitLister`] of
//! `toolbox-services`.
//!
//! [`watch`] asks for a fresh list on every systemd signal, so the
//! page follows changes made anywhere, in this app or outside it.

#[cfg(test)]
mod tests;
pub mod watch;

use std::sync::Arc;
use std::time::Duration;

use tokio::runtime::Handle;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::sync::watch as channel;
use toolbox_core::features::services::UnitInfo;
use toolbox_gui_kit::Loadable;
use toolbox_plan::UnitScope;
pub use toolbox_services::units::{Systemd, UnitList, UnitLister};
use toolbox_system::CallError;
use tracing::Instrument as _;

use crate::reader::fresh;
use crate::spawn::spawn_logged;
use crate::waking::Wake;

/// A unit change comes with a burst of signals (job, properties, files).
/// One list after the burst is enough.
pub const SETTLE: Duration = Duration::from_millis(300);

/// Why a list failed, as [`Units`] keeps it. The page words it, the
/// reason itself goes to the log.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitsFailure {
    /// No systemd on the bus, or no bus.
    Unavailable,
    Denied,
    Failed,
}

pub type UnitListing = Loadable<Vec<UnitInfo>, UnitsFailure>;

#[derive(Clone, Debug, Default)]
pub struct Units {
    pub system: UnitListing,
    pub user: UnitListing,
}

/// Cloned into the watchers.
#[derive(Clone, Debug)]
pub struct UnitRequests(UnboundedSender<UnitScope>);

#[derive(Debug)]
pub struct UnitsHandle {
    requests: UnitRequests,
    units: channel::Receiver<Units>,
}

impl Units {
    pub const fn get(&self, scope: UnitScope) -> &UnitListing {
        match scope {
            UnitScope::System => &self.system,
            UnitScope::User => &self.user,
        }
    }

    const fn get_mut(&mut self, scope: UnitScope) -> &mut UnitListing {
        match scope {
            UnitScope::System => &mut self.system,
            UnitScope::User => &mut self.user,
        }
    }

    pub fn finish(&mut self, scope: UnitScope, list: UnitList) {
        let units = self.get_mut(scope);
        match list {
            Ok(list) => units.set_loaded(list),
            Err(error) => units.fail(failure(scope, &error)),
        }
    }
}

impl UnitRequests {
    pub fn refresh(&self, scope: UnitScope) {
        if self.0.send(scope).is_err() {
            tracing::debug!("the units service is gone");
        }
    }
}

impl UnitsHandle {
    pub fn refresh(&self, scope: UnitScope) {
        self.requests.refresh(scope);
    }

    pub fn requests(&self) -> UnitRequests {
        self.requests.clone()
    }

    pub fn changed(&mut self) -> Option<Units> {
        fresh(&mut self.units)
    }
}

/// Starts the service and the first list of each scope. Requests within
/// `settle` of each other become one list per scope.
#[must_use = "the handle is the only way to the service"]
pub fn spawn(
    runtime: &Handle,
    wake: Wake,
    lister: Arc<dyn UnitLister>,
    settle: Duration,
) -> UnitsHandle {
    let (sender, receiver) = unbounded_channel();
    let (lists, units) = channel::channel(Units::default());
    let handle = UnitsHandle {
        requests: UnitRequests(sender),
        units,
    };

    handle.refresh(UnitScope::System);
    handle.refresh(UnitScope::User);
    spawn_logged(
        runtime,
        "unit lists",
        serve(receiver, lister, lists, wake, settle),
    );
    handle
}

async fn serve(
    mut requests: UnboundedReceiver<UnitScope>,
    lister: Arc<dyn UnitLister>,
    outlet: channel::Sender<Units>,
    wake: Wake,
    settle: Duration,
) {
    while let Some(first) = requests.recv().await {
        tokio::time::sleep(settle).await;

        let mut scopes = vec![first];
        while let Ok(scope) = requests.try_recv() {
            if !scopes.contains(&scope) {
                scopes.push(scope);
            }
        }

        outlet.send_modify(|units| {
            for scope in &scopes {
                units.get_mut(*scope).start_loading();
            }
        });
        wake.wake();

        for scope in scopes {
            let span = tracing::debug_span!("read", what = "units", scope = scope.as_str());
            let list = lister.list(scope).instrument(span).await;

            outlet.send_modify(|units| units.finish(scope, list));
            wake.wake();
        }
    }
}

/// The page words only the kind of failure, so the reason goes to the
/// log.
fn failure(scope: UnitScope, error: &CallError) -> UnitsFailure {
    match error {
        CallError::Unavailable => UnitsFailure::Unavailable,
        CallError::Denied => UnitsFailure::Denied,
        CallError::Failed(_) | CallError::Bus { error: _ } => {
            let reason = error.reason().unwrap_or_default();
            tracing::warn!("listing the {} units: {reason}", scope.as_str());
            UnitsFailure::Failed
        }
    }
}
