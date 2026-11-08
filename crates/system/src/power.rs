//! Restarting the computer through logind, which asks polkit itself, and
//! holding a shutdown back while an apply runs. No root helper is
//! involved.

#[cfg(test)]
mod tests;

use toolbox_plan::UnitScope;

use crate::bus::{CallError, Method, call, connect};

pub const REBOOT: Method = Method {
    destination: "org.freedesktop.login1",
    path: "/org/freedesktop/login1",
    interface: "org.freedesktop.login1.Manager",
    name: "Reboot",
};

pub const INHIBIT: Method = Method {
    destination: "org.freedesktop.login1",
    path: "/org/freedesktop/login1",
    interface: "org.freedesktop.login1.Manager",
    name: "Inhibit",
};

/// `Reboot(interactive)`.
pub type RebootBody = (bool,);

/// Holds shutdown back until it closes.
pub type ShutdownLock = zbus::OwnedFd;

/// `Inhibit(what, who, why, mode)`.
pub type InhibitBody = (&'static str, &'static str, String, &'static str);

/// Interactive, so polkit may show a password prompt when other users are
/// logged in or the policy wants one.
pub const fn reboot_body() -> RebootBody {
    (true,)
}

pub fn inhibit_body(why: &str) -> InhibitBody {
    ("shutdown:sleep", "Garuda Toolbox", why.to_owned(), "block")
}

/// Keeps shutdown and sleep back while the returned descriptor is open,
/// so a restart from the desktop's menu waits for pacman. logind lets an
/// active session's user take it without a password.
///
/// # Errors
///
/// The failure to reach logind or its refusal.
pub async fn inhibit_shutdown(why: &str) -> Result<ShutdownLock, CallError> {
    let connection = connect(UnitScope::System).await?;
    call(&connection, &INHIBIT, &inhibit_body(why)).await
}
