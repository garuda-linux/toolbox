//! Live updates: one watcher per bus asks for a fresh list on every
//! systemd signal. Without a bus the page still lists on request, only
//! live updates wait for the bus to come back.

#[cfg(test)]
mod tests;

use std::time::Duration;

use futures_lite::StreamExt as _;
use tokio::runtime::Handle;
use tokio::time::Instant;
use toolbox_plan::UnitScope;
use toolbox_system::systemd::{SUBSCRIBE, SYSTEMD_DESTINATION};
use toolbox_system::{CallError, call, classify, connect};
use zbus::{MatchRule, MessageStream};

use super::UnitRequests;

/// Live updates retry this long between attempts, doubling each time.
const FIRST_RETRY: Duration = Duration::from_secs(1);
const MAX_RETRY: Duration = Duration::from_secs(30);
/// A stream that lived this long was a working connection, so its end is
/// a new outage: the waits start short again and it is reported once more.
/// Twice the longest wait, so a bus that drops every stream soon after it
/// connects keeps the long waits and stays quiet.
const HEALTHY: Duration = Duration::from_secs(60);

#[derive(Debug)]
struct Backoff {
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
    fn after(&mut self, ran: Duration) -> (Duration, bool) {
        if ran >= HEALTHY {
            *self = Self::default();
        }

        let step = (self.next, self.reported);
        self.next = (self.next * 2).min(MAX_RETRY);
        self.reported = true;
        step
    }
}

/// Starts a watcher for each bus. They run as long as the app.
pub fn spawn(runtime: &Handle, requests: &UnitRequests) {
    for scope in [UnitScope::System, UnitScope::User] {
        let requests = requests.clone();
        drop(runtime.spawn(async move {
            watch_with(scope, &requests, || live(scope, requests.clone())).await;
        }));
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

/// One live stream: subscribes and reports signals until the bus drops.
/// A fresh stream lists again, so a reconnect never shows stale units.
async fn live(scope: UnitScope, requests: UnitRequests) -> Result<(), CallError> {
    let connection = connect(scope).await?;

    // systemd only sends its manager signals to subscribed clients.
    call::<_, ()>(&connection, &SUBSCRIBE, &()).await?;

    let rule = MatchRule::builder()
        .msg_type(zbus::message::Type::Signal)
        .sender(SYSTEMD_DESTINATION)
        .build()
        .map_err(|error| classify(&error))?;

    let mut signals = MessageStream::for_match_rule(rule, &connection, None)
        .await
        .map_err(|error| classify(&error))?;

    requests.refresh(scope);
    while signals.next().await.is_some() {
        requests.refresh(scope);
    }

    Ok(())
}
