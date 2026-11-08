//! Restarting the computer through logind, which asks polkit itself.

use std::fmt::Debug;

use crossbeam_channel::Sender;
use tokio::runtime::Handle;
use toolbox_plan::UnitScope;
use toolbox_system::{CallError, REBOOT, classify, connect, reboot_body};

/// Restarts the machine. Tests and dry runs restart nothing.
pub trait Power: Debug + Send + Sync {
    /// Asks for the restart. Only a refusal comes back, through `refused`:
    /// a restart that works ends the app.
    fn restart(&self, refused: Sender<CallError>);
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
    fn restart(&self, refused: Sender<CallError>) {
        drop(self.runtime.spawn(async move {
            if let Err(error) = reboot().await {
                tracing::warn!("no restart: {error}");

                // An error only means the app is shutting down.
                refused.send(error).ok();
            }
        }));
    }
}

impl Power for DryRunPower {
    fn restart(&self, _refused: Sender<CallError>) {
        tracing::info!("dry run: the computer would restart now");
    }
}

/// polkit may show its dialog for as long as the user takes, so the call
/// has no deadline.
async fn reboot() -> Result<(), CallError> {
    let connection = connect(UnitScope::System).await?;

    connection
        .call_method(
            Some(REBOOT.destination),
            REBOOT.path,
            Some(REBOOT.interface),
            REBOOT.name,
            &reboot_body(),
        )
        .await
        .map(drop)
        .map_err(|error| classify(&error))
}
