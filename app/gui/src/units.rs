//! The unit lists of the Services and Network pages, from the
//! [`UnitLister`] of `toolbox-services`.
//!
//! Nothing is listed or watched before one of these pages opened. Then
//! [`watch`] asks for a fresh list on every change of the unit set, so
//! the page follows changes made anywhere, in this app or outside it.

mod edits;
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
pub(crate) use watch::Backoff;

use crate::reader::fresh;
use crate::spawn::spawn_logged;
use crate::waking::Wake;

/// A unit change comes with a burst of signals (unit, job, files). One
/// list after the burst is enough.
pub const SETTLE: Duration = Duration::from_millis(300);

/// Why a list failed, as [`UnitLists`] keeps it. The page words it, the
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
pub struct UnitLists {
    pub system: UnitListing,
    pub user: UnitListing,
}

/// The Services page's service: the last lists and the task that makes
/// them.
#[derive(Debug, Default)]
pub struct Units {
    lists: UnitLists,
    /// `None` in tests that need no lists: nothing is listed.
    handle: Option<UnitsHandle>,
    /// A control or a journal did not start, as another plan runs. It
    /// stays on the page until dismissed or the next one starts.
    refused: bool,
}

/// Cloned into the watchers.
#[derive(Clone, Debug)]
pub struct UnitRequests {
    sender: UnboundedSender<Request>,
    /// Whether a page that reads units opened.
    opened: channel::Receiver<bool>,
}

#[derive(Debug)]
pub struct UnitsHandle {
    requests: UnitRequests,
    units: channel::Receiver<UnitLists>,
    opened: channel::Sender<bool>,
}

/// A list asked for. One for a signal waits out the burst for
/// [`SETTLE`], the page's first look and Retry list at once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Request {
    scope: UnitScope,
    settle: bool,
}

impl Units {
    pub const fn new(handle: Option<UnitsHandle>) -> Self {
        Self {
            lists: UnitLists {
                system: Loadable::NotLoaded,
                user: Loadable::NotLoaded,
            },
            handle,
            refused: false,
        }
    }

    /// The lists a test hands the page, without a task.
    #[cfg(any(test, feature = "testing"))]
    pub const fn showing(lists: UnitLists) -> Self {
        Self {
            lists,
            handle: None,
            refused: false,
        }
    }

    pub const fn refused(&self) -> bool {
        self.refused
    }

    /// Whether App sent the last control or journal, which runs at once.
    pub const fn set_refused(&mut self, refused: bool) {
        self.refused = refused;
    }

    pub const fn get(&self, scope: UnitScope) -> &UnitListing {
        self.lists.get(scope)
    }

    pub fn refresh(&self, scope: UnitScope) {
        if let Some(handle) = &self.handle {
            handle.refresh(scope);
        }
    }

    /// A page that shows units opened. The first one lists both scopes
    /// and starts the watchers.
    pub fn open(&self) {
        if let Some(handle) = &self.handle {
            handle.open();
        }
    }

    /// Whether a page opened and a list it asked for has not been taken.
    #[cfg(any(test, feature = "testing"))]
    pub fn awaits(&self) -> bool {
        let opened = self
            .handle
            .as_ref()
            .is_some_and(|handle| *handle.opened.borrow());

        let pending = |list: &UnitListing| {
            matches!(
                list,
                Loadable::NotLoaded | Loadable::Loading { previous: _ }
            )
        };

        opened && (pending(&self.lists.system) || pending(&self.lists.user))
    }

    /// The lists that came in since the last frame.
    pub fn take(&mut self) {
        if let Some(lists) = self.handle.as_mut().and_then(UnitsHandle::changed) {
            self.lists = lists;
        }
    }
}

impl UnitLists {
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
    /// For a signal, which comes in a burst.
    pub fn refresh(&self, scope: UnitScope) {
        self.send(Request {
            scope,
            settle: true,
        });
    }

    /// Returns once a page that reads units opened.
    pub async fn opened(&self) {
        let mut opened = self.opened.clone();
        let closed = opened.wait_for(|open| *open).await.is_err();
        if closed {
            // The app ends, and no page opens anymore.
            std::future::pending::<()>().await;
        }
    }

    fn now(&self, scope: UnitScope) {
        self.send(Request {
            scope,
            settle: false,
        });
    }

    fn send(&self, request: Request) {
        if self.sender.send(request).is_err() {
            tracing::debug!("the units service is gone");
        }
    }
}

impl UnitsHandle {
    /// Retry and the end of a plan. Before a page opened nothing shows
    /// the lists, so nothing is read.
    pub fn refresh(&self, scope: UnitScope) {
        if *self.opened.borrow() {
            self.requests.now(scope);
        }
    }

    pub fn open(&self) {
        if self
            .opened
            .send_if_modified(|open| !std::mem::replace(open, true))
        {
            self.requests.now(UnitScope::System);
            self.requests.now(UnitScope::User);
        }
    }

    pub fn requests(&self) -> UnitRequests {
        self.requests.clone()
    }

    pub fn changed(&mut self) -> Option<UnitLists> {
        fresh(&mut self.units)
    }
}

/// Starts the service, which lists once [`UnitsHandle::open`] was called.
/// A signal's request waits `settle`, and the requests within it become
/// one list per scope.
#[must_use = "the handle is the only way to the service"]
pub fn spawn(
    runtime: &Handle,
    wake: Wake,
    lister: Arc<dyn UnitLister>,
    settle: Duration,
) -> UnitsHandle {
    let (sender, receiver) = unbounded_channel();
    let (lists, units) = channel::channel(UnitLists::default());
    let (opened, watched) = channel::channel(false);
    let handle = UnitsHandle {
        requests: UnitRequests {
            sender,
            opened: watched,
        },
        units,
        opened,
    };

    spawn_logged(
        runtime,
        "unit lists",
        serve(receiver, lister, lists, wake, settle),
    );
    handle
}

async fn serve(
    mut requests: UnboundedReceiver<Request>,
    lister: Arc<dyn UnitLister>,
    outlet: channel::Sender<UnitLists>,
    wake: Wake,
    settle: Duration,
) {
    while let Some(first) = requests.recv().await {
        if first.settle {
            tokio::time::sleep(settle).await;
        }

        let mut scopes = vec![first.scope];
        while let Ok(next) = requests.try_recv() {
            if !scopes.contains(&next.scope) {
                scopes.push(next.scope);
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
