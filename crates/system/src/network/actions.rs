//! What the Network page does to the machine outside a plan: switch a
//! radio and make NetworkManager check the way to the internet. The
//! hotspot tool starts through the `Launcher` port like every other tool.
//! A dry run (the self-test) does none of it.

#[cfg(test)]
mod tests;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use toolbox_state::network::{Connectivity, RadioTarget};

use super::online::connectivity;
use crate::{NetworkSource, ProbeError};

pub type Checked = Pin<Box<dyn Future<Output = Connectivity> + Send>>;

pub trait NetworkActions: Send + Sync {
    /// The way to the internet. `recheck` asks NetworkManager to fetch
    /// its test page first.
    fn connectivity(&self, recheck: bool) -> Checked;

    /// Blocks or unblocks a radio. Blocks, so call it off the GUI thread.
    ///
    /// # Errors
    fn set_radio(&self, target: RadioTarget, blocked: bool) -> Result<(), ProbeError>;
}

pub struct SystemActions {
    network: Arc<dyn NetworkSource>,
}

impl SystemActions {
    pub fn new(network: Arc<dyn NetworkSource>) -> Self {
        Self { network }
    }
}

impl NetworkActions for SystemActions {
    fn connectivity(&self, recheck: bool) -> Checked {
        Box::pin(connectivity(recheck))
    }

    fn set_radio(&self, target: RadioTarget, blocked: bool) -> Result<(), ProbeError> {
        self.network.set_radio(target, blocked)
    }
}
