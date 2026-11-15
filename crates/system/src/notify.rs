//! Desktop notifications through `org.freedesktop.Notifications`, which
//! every desktop with a notification daemon offers.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use zbus::Variant;

use crate::bus::Method;

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
