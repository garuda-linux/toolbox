//! The manager's signals that change what the unit lists show, followed on
//! the bus of a scope. The connection is the one the lists share, and it is
//! dropped when the stream ends, so the next follow connects again.

#[cfg(test)]
mod tests;

use futures_lite::StreamExt as _;
use futures_lite::future::Boxed;
use toolbox_plan::UnitScope;
use zbus::{Connection, MatchRule, MessageStream};

use super::{MANAGER_INTERFACE, MANAGER_PATH, SUBSCRIBE, SYSTEMD_DESTINATION};
use crate::bus::{CallError, call, classify, forget_shared, shared};

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

/// Called for each signal that relists.
pub type OnSignal = Box<dyn FnMut() + Send>;

pub trait UnitSignals: Send + Sync {
    /// Subscribes to the relisting signals of `scope` and calls `on_signal`
    /// for each one, until the bus drops the subscription.
    ///
    /// # Errors
    ///
    /// The failure to reach the bus or to subscribe, classified.
    fn follow(&self, scope: UnitScope, on_signal: OnSignal) -> Boxed<Result<(), CallError>>;
}

#[derive(Clone, Copy, Debug)]
pub struct SystemUnitSignals;

impl UnitSignals for SystemUnitSignals {
    fn follow(&self, scope: UnitScope, on_signal: OnSignal) -> Boxed<Result<(), CallError>> {
        Box::pin(async move {
            let ended = follow(scope, on_signal).await;
            forget_shared(scope);
            ended
        })
    }
}

async fn follow(scope: UnitScope, mut on_signal: OnSignal) -> Result<(), CallError> {
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

    on_signal();
    while signals.next().await.is_some() {
        on_signal();
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
