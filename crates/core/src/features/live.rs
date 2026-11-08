//! What the live ISO offers on Home: installing Garuda Linux and entering
//! an installed system to repair it.

#[cfg(test)]
mod tests;

use toolbox_plan::{Plan, RootAction};

/// The ISO's own installer launcher. It asks for root itself, like the
/// installer icon on the ISO's desktop, so the app starts it as the user.
pub const INSTALLER: &str = "/usr/lib/garuda/pkexec-gui";
pub const INSTALLER_ARGS: [&str; 1] = ["calamares"];

#[must_use = "a plan does nothing until it is run"]
pub fn plan_chroot() -> Plan {
    Plan::root(RootAction::ChrootInstallation)
}
