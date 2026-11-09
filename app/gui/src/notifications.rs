//! Desktop notifications through `org.freedesktop.Notifications`. The app
//! sends one when a run ends while the window is in the background.

use std::fmt::Debug;

use tokio::runtime::Handle;
use toolbox_system::{CallError, NOTIFY, Notification, call, classify, notify_body};
use zbus::Connection;

use crate::spawn::spawn_logged;

/// Shows notifications. Tests record them instead.
pub trait Notifier: Debug + Send + Sync {
    fn show(&self, notification: Notification);
}

#[derive(Debug)]
pub struct DesktopNotifier {
    runtime: Handle,
}

impl DesktopNotifier {
    pub const fn new(runtime: Handle) -> Self {
        Self { runtime }
    }
}

impl Notifier for DesktopNotifier {
    fn show(&self, notification: Notification) {
        spawn_logged(&self.runtime, "notification", async move {
            // Without a notification daemon the bubble is only logged.
            if let Err(error) = send(&notification).await {
                tracing::info!("no desktop notification: {error}");
            }
        });
    }
}

async fn send(notification: &Notification) -> Result<(), CallError> {
    let connection = Connection::session()
        .await
        .map_err(|error| classify(&error))?;

    // The daemon answers with the id of the new notification.
    call::<_, u32>(&connection, &NOTIFY, &notify_body(notification))
        .await
        .map(drop)
}
