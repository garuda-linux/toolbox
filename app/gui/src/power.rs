//! Restarting the computer through logind, which asks polkit itself.

use std::fmt::Debug;

use tokio::runtime::Handle;
use toolbox_plan::UnitScope;
use toolbox_system::{
    CallError, INTERACTIVE_TIMEOUT, REBOOT, call_with_timeout, connect, reboot_body,
};

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
}

/// Logs the restart instead, for the dry run and its self-test.
#[derive(Debug)]
pub struct DryRunPower;

impl Logind {
    pub const fn new(runtime: Handle) -> Self {
        Self { runtime }
    }
}

impl Power for Logind {
    fn restart(&self, refused: Waking<CallError>) {
        spawn_logged(&self.runtime, "restart", async move {
            if let Err(error) = reboot().await {
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

async fn reboot() -> Result<(), CallError> {
    let connection = connect(UnitScope::System).await?;

    call_with_timeout(&connection, &REBOOT, &reboot_body(), INTERACTIVE_TIMEOUT).await
}
