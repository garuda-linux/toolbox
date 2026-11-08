//! A shutdown or sleep in the middle of pacman can leave the system
//! unbootable, so logind holds both back while a plan runs.

#[cfg(test)]
mod tests;

use toolbox_plan::protocol::Mode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inhibitor {
    /// Asks logind for a `block` lock, as the app and the command line do.
    Logind,
    /// Takes no lock, for tests.
    Off,
}

/// Open while the plan runs. logind drops the lock when it closes.
pub struct Hold {
    _lock: Option<toolbox_system::ShutdownLock>,
}

impl Inhibitor {
    /// A dry run changes nothing, so it holds nothing back. A refused
    /// lock is logged and the plan runs anyway: losing the apply would be
    /// worse than an unguarded one.
    pub async fn hold(self, mode: Mode) -> Hold {
        if self == Self::Off || mode == Mode::DryRun {
            return Hold { _lock: None };
        }

        match toolbox_system::inhibit_shutdown("Changes are being applied").await {
            Ok(lock) => Hold { _lock: Some(lock) },
            Err(error) => {
                tracing::warn!("cannot hold shutdown back during the apply: {error}");
                Hold { _lock: None }
            }
        }
    }
}
