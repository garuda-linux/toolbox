//! D-Bus methods the app calls, how their failures are sorted, and the
//! bounded call itself.

mod client;
#[cfg(test)]
mod tests;

use std::io;

pub use client::{CALL_TIMEOUT, call, connect};
use zbus::DBusError as _;

const UNAVAILABLE: [&str; 2] = [
    "org.freedesktop.DBus.Error.ServiceUnknown",
    "org.freedesktop.DBus.Error.NameHasNoOwner",
];

/// polkit answers a dismissed or refused password prompt with one of these,
/// which the user caused and the app shows differently from a failure.
const DENIED: [&str; 3] = [
    "org.freedesktop.DBus.Error.AccessDenied",
    "org.freedesktop.DBus.Error.InteractiveAuthorizationRequired",
    "org.freedesktop.DBus.Error.AuthFailed",
];

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CallError {
    /// No bus, or no service on it, for example a desktop without a
    /// notification daemon.
    #[error("not available on this system")]
    Unavailable,
    /// The service refused, usually because polkit did not authorize it.
    #[error("not authorized")]
    Denied,
    #[error("call failed: {0}")]
    Failed(String),
}

pub struct Method {
    pub destination: &'static str,
    pub path: &'static str,
    pub interface: &'static str,
    pub name: &'static str,
}

#[must_use = "the classified error is lost otherwise"]
pub fn classify(error: &zbus::Error) -> CallError {
    if let zbus::Error::MethodError(name, description, _) = error {
        return from_name(name.as_str(), description.as_deref());
    }

    if let zbus::Error::FDO(fdo) = error {
        return from_name(fdo.name().as_str(), fdo.description());
    }

    if let zbus::Error::Address(_) = error {
        return CallError::Unavailable;
    }

    if let zbus::Error::InputOutput(io_error) | zbus::Error::Connection(io_error, _) = error
        && matches!(
            io_error.kind(),
            io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused
        )
    {
        return CallError::Unavailable;
    }

    CallError::Failed(error.to_string())
}

fn from_name(name: &str, description: Option<&str>) -> CallError {
    if UNAVAILABLE.contains(&name) {
        CallError::Unavailable
    } else if DENIED.contains(&name) {
        CallError::Denied
    } else {
        CallError::Failed(description.map_or_else(
            || name.to_owned(),
            |description| format!("{name}: {description}"),
        ))
    }
}
