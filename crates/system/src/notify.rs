//! Desktop notifications through `org.freedesktop.Notifications`, which
//! every desktop with a notification daemon offers.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fmt::Debug;

use tokio::runtime::Handle;
use zbus::{Connection, Variant};

use crate::bus::{CallError, Method, call, classify};

pub const NOTIFY: Method = Method {
    destination: "org.freedesktop.Notifications",
    path: "/org/freedesktop/Notifications",
    interface: "org.freedesktop.Notifications",
    name: "Notify",
};

const APP_NAME: &str = "Garuda Toolbox";
const APP_ICON: &str = "garuda-toolbox";

/// The daemon decides how long the bubble stays.
const EXPIRE_DEFAULT: i32 = -1;

/// `Notify(app_name, replaces_id, app_icon, summary, body, actions, hints,
/// expire_timeout)`.
pub type NotifyBody<'notification> = (
    &'static str,
    u32,
    &'static str,
    &'notification str,
    &'notification str,
    Vec<&'static str>,
    BTreeMap<&'static str, Variant<'static>>,
    i32,
);

/// One bubble. The texts come translated from the UI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notification {
    pub summary: String,
    pub body: String,
}

/// Shows notifications. Tests record them instead.
pub trait Notifier: Debug + Send + Sync {
    fn show(&self, notification: Notification);
}

/// Sends each notification to the session's daemon on `runtime`.
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
        drop(self.runtime.spawn(async move {
            // Without a notification daemon the bubble is only logged.
            if let Err(error) = send(&notification).await {
                tracing::info!("no desktop notification: {error}");
            }
        }));
    }
}

pub fn notify_body(notification: &Notification) -> NotifyBody<'_> {
    (
        APP_NAME,
        0_u32,
        APP_ICON,
        &notification.summary,
        &notification.body,
        Vec::new(),
        BTreeMap::new(),
        EXPIRE_DEFAULT,
    )
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
