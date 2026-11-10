//! D-Bus methods the app calls, how their failures are sorted, and the
//! bounded call itself.

mod client;
#[cfg(test)]
mod tests;

use std::io;

pub use client::{CALL_TIMEOUT, INTERACTIVE_TIMEOUT, call, call_with_timeout, connect};
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

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
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
    /// The bus's own error, kept as the source.
    #[error("call failed: {error}")]
    Bus {
        #[source]
        error: zbus::Error,
    },
}

/// zbus compares its errors without claiming `Eq`. The other variants
/// are plain values.
impl Eq for CallError {}

impl CallError {
    /// What went wrong in words, for a log line.
    pub fn reason(&self) -> Option<String> {
        match self {
            Self::Unavailable | Self::Denied => None,
            Self::Failed(reason) => Some(reason.clone()),
            Self::Bus { error } => Some(error.to_string()),
        }
    }
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

    CallError::Bus {
        error: error.clone(),
    }
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
