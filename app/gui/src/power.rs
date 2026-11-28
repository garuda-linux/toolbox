//! Restarting the computer through logind, which asks polkit itself.

use std::fmt::Debug;
use std::sync::Arc;

use tokio::runtime::Handle;
use toolbox_system::{CallError, Restart};

use crate::spawn::spawn_logged;
use crate::waking::Waking;

/// Restarts the machine. Tests and dry runs restart nothing.
pub trait Power: Debug + Send + Sync {
    /// Asks for the restart. Only a refusal comes back, through `refused`,
    /// which wakes the window: a restart that works ends the app.
    fn restart(&self, refused: Waking<CallError>);
}

#[derive(Debug)]
pub struct Logind {
    runtime: Handle,
    restart: Arc<dyn Restart>,
}

/// Logs the restart instead, for the dry run and its self-test.
#[derive(Debug)]
pub struct DryRunPower;

impl Logind {
    pub fn new(runtime: Handle, restart: Arc<dyn Restart>) -> Self {
        Self { runtime, restart }
    }
}

impl Power for Logind {
    fn restart(&self, refused: Waking<CallError>) {
        let restart = Arc::clone(&self.restart);
        spawn_logged(&self.runtime, "restart", async move {
            if let Err(error) = restart.restart().await {
                tracing::warn!("no restart: {error}");
                refused.send(error);
            }
        });
    }
}

impl Power for DryRunPower {
    fn restart(&self, _refused: Waking<CallError>) {
        tracing::info!("dry run: the computer would restart now");
    }
}
