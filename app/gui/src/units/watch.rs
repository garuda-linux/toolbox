//! Live updates: one watcher per bus asks for a fresh list whenever the
//! set of units or their files change.
//!
//! Without a bus the page still lists on request, only live updates wait
//! for the bus to come back. The watchers start once a page that shows
//! units opened.

#[cfg(test)]
mod tests;

use std::time::Duration;

use futures_lite::StreamExt as _;
use tokio::runtime::Handle;
use tokio::time::Instant;
use toolbox_plan::UnitScope;
use toolbox_system::systemd::{MANAGER_INTERFACE, MANAGER_PATH, SUBSCRIBE, SYSTEMD_DESTINATION};
use toolbox_system::{CallError, call, classify, forget_shared, shared};
use zbus::{Connection, MatchRule, MessageStream};

use super::UnitRequests;
use crate::spawn::spawn_logged;

/// The manager's signals that change what the lists show.
///
/// A unit loaded or gone, a job's end (a start, stop or restart), changed
/// unit files and a reload. Property changes of single units, which
/// transient scopes and timers send all the time, list nothing.
pub const RELISTING: [&str; 5] = [
    "UnitNew",
    "UnitRemoved",
    "JobRemoved",
    "UnitFilesChanged",
    "Reloading",
];

/// Live updates retry this long between attempts, doubling each time.
const FIRST_RETRY: Duration = Duration::from_secs(1);
const MAX_RETRY: Duration = Duration::from_secs(30);
/// A stream that lived this long was a working connection, so its end is
/// a new outage: the waits start short again and it is reported once more.
/// Twice the longest wait, so a bus that drops every stream soon after it
/// connects keeps the long waits and stays quiet.
const HEALTHY: Duration = Duration::from_secs(60);

#[derive(Debug)]
pub(crate) struct Backoff {
    next: Duration,
    reported: bool,
}

impl Default for Backoff {
    fn default() -> Self {
        Self {
            next: FIRST_RETRY,
            reported: false,
        }
    }
}

impl Backoff {
    /// After a stream that ran for `ran`: the wait before the next one,
    /// and whether this end repeats an outage already reported.
    pub(crate) fn after(&mut self, ran: Duration) -> (Duration, bool) {
        if ran >= HEALTHY {
            *self = Self::default();
        }

        let step = (self.next, self.reported);
        self.next = (self.next * 2).min(MAX_RETRY);
        self.reported = true;
        step
    }
}

/// Starts a watcher for each bus, which waits for the first page that
/// shows units. They run as long as the app.
pub fn spawn(runtime: &Handle, requests: &UnitRequests) {
    for scope in [UnitScope::System, UnitScope::User] {
        let requests = requests.clone();
        spawn_logged(runtime, "unit watch", async move {
            requests.opened().await;
            watch_with(scope, &requests, || live(scope, requests.clone())).await;
        });
    }
}

async fn watch_with<F, S>(scope: UnitScope, requests: &UnitRequests, mut live: F)
where
    F: FnMut() -> S,
    S: Future<Output = Result<(), CallError>>,
{
    let mut backoff = Backoff::default();

    loop {
        let started = Instant::now();
        let ended = live().await;
        let (wait, repeat) = backoff.after(started.elapsed());
        let note = match ended {
            Ok(()) => format!("the {} unit signals ended, watching again", scope.as_str()),
            Err(error) => format!(
                "no live updates for the {} units ({error}), retrying",
                scope.as_str()
            ),
        };

        // tracing takes no level from a variable.
        if repeat {
            tracing::debug!("{note}");
        } else {
            tracing::warn!("{note}");
        }

        // The stream broke on a stale list, so ask for a fresh one.
        requests.refresh(scope);
        tokio::time::sleep(wait).await;
    }
}

/// One live stream on the connection the lists share: subscribes and
/// reports signals until the bus drops. A fresh stream lists again, so a
/// reconnect never shows stale units. Its end drops the connection, so
/// the next one connects again.
async fn live(scope: UnitScope, requests: UnitRequests) -> Result<(), CallError> {
    let ended = follow(scope, &requests).await;
    forget_shared(scope);
    ended
}

async fn follow(scope: UnitScope, requests: &UnitRequests) -> Result<(), CallError> {
    let connection = shared(scope).await?;

    // systemd only sends its manager signals to subscribed clients.
    call::<_, ()>(&connection, &SUBSCRIBE, &()).await?;

    let [
        unit_new,
        unit_removed,
        job_removed,
        files_changed,
        reloading,
    ] = RELISTING;

    let mut signals = manager_signal(&connection, unit_new)
        .await?
        .or(manager_signal(&connection, unit_removed).await?)
        .or(manager_signal(&connection, job_removed).await?)
        .or(manager_signal(&connection, files_changed).await?)
        .or(manager_signal(&connection, reloading).await?);

    requests.refresh(scope);
    while signals.next().await.is_some() {
        requests.refresh(scope);
    }

    Ok(())
}

/// The manager's signal `member`, matched by the bus, so other signals
/// never wake the app.
async fn manager_signal(
    connection: &Connection,
    member: &'static str,
) -> Result<MessageStream, CallError> {
    let rule = MatchRule::builder()
        .msg_type(zbus::message::Type::Signal)
        .sender(SYSTEMD_DESTINATION)
        .path(MANAGER_PATH)
        .interface(MANAGER_INTERFACE)
        .member(member)
        .build()
        .map_err(|error| classify(&error))?;

    MessageStream::for_match_rule(rule, connection, None)
        .await
        .map_err(|error| classify(&error))
}
