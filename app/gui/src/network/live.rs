//! Live interface cards: NetworkManager's property signals ask for a fresh
//! read, and a burst of them settles into one read.

use std::sync::Arc;
use std::time::Duration;

use futures_lite::StreamExt as _;
use tokio::runtime::Handle;
use tokio::sync::Notify;
use tokio::time::Instant;
use toolbox_plan::UnitScope;
use toolbox_system::{CallError, NetworkActions, forget_shared, property_changes, shared};

use super::jobs::{Landed, cards_or_none};
use crate::spawn::spawn_logged;
use crate::units::Backoff;
use crate::waking::Waking;

/// A burst of signals waits this long, so one read answers all of them.
const SETTLE: Duration = Duration::from_millis(500);

/// Starts the watcher, which runs as long as the app and sends each read
/// of the cards to `answers`.
pub fn spawn(runtime: &Handle, actions: Arc<dyn NetworkActions>, answers: Waking<Landed>) {
    let changed = Arc::new(Notify::new());
    spawn_logged(runtime, "network watch", watch(Arc::clone(&changed)));
    spawn_logged(runtime, "network read", reread(changed, actions, answers));
}

/// Follows the signals and waits again after each stream's end, with the
/// units' backoff. A permit of `changed` is kept while a read runs, so a
/// signal in between asks for one more read.
async fn watch(changed: Arc<Notify>) {
    let mut backoff = Backoff::default();

    loop {
        let started = Instant::now();
        let ended = follow(&changed).await;
        let (wait, repeat) = backoff.after(started.elapsed());
        let note = match &ended {
            Ok(()) => "the network signals ended, watching again".to_owned(),
            Err(error) => format!("no live updates for the interface cards ({error}), retrying"),
        };

        // tracing takes no level from a variable.
        if repeat {
            tracing::debug!("{note}");
        } else {
            tracing::warn!("{note}");
        }

        // A stream that ended may have missed a change, so the cards are read again.
        changed.notify_one();
        tokio::time::sleep(wait).await;
    }
}

/// One live stream on the connection the units share. Its end drops the
/// connection, so the next stream connects again.
async fn follow(changed: &Notify) -> Result<(), CallError> {
    let ended = stream(changed).await;
    forget_shared(UnitScope::System);
    ended
}

async fn stream(changed: &Notify) -> Result<(), CallError> {
    let connection = shared(UnitScope::System).await?;
    let mut signals = property_changes(&connection).await?;

    while signals.next().await.is_some() {
        changed.notify_one();
    }

    Ok(())
}

async fn reread(changed: Arc<Notify>, actions: Arc<dyn NetworkActions>, answers: Waking<Landed>) {
    loop {
        changed.notified().await;
        tokio::time::sleep(SETTLE).await;

        let cards = cards_or_none(actions.interface_cards().await);
        answers.send(Landed::Interfaces(cards));
    }
}
