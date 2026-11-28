//! A desktop portal that answers the settings it is given and never reports
//! a change.

use futures_lite::future::Boxed;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

use crate::look_portal::{LookPortal, PortalSettings};

#[derive(Clone, Debug, Default)]
pub struct FixedLook(pub PortalSettings);

impl LookPortal for FixedLook {
    fn read(&self) -> Boxed<PortalSettings> {
        let settings = self.0;
        Box::pin(async move { settings })
    }

    fn changes(&self) -> Boxed<UnboundedReceiver<()>> {
        let (sender, receiver) = unbounded_channel();
        drop(sender);
        Box::pin(async move { receiver })
    }
}
