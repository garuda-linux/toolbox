use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use toolbox_state::network::{DefaultRoute, Device, Radio, RadioAccess, RadioTarget};

use super::{NetworkActions as _, SystemActions};
use crate::{NetworkSource, ProbeError};

#[derive(Default)]
struct Switches(AtomicUsize);

impl NetworkSource for Switches {
    fn default_route(&self) -> Result<DefaultRoute, ProbeError> {
        Err(ProbeError::Unavailable)
    }

    fn radios(&self) -> Result<Vec<Radio>, ProbeError> {
        Err(ProbeError::Unavailable)
    }

    fn radio_access(&self) -> RadioAccess {
        RadioAccess::Missing
    }

    fn set_radio(&self, _target: RadioTarget, _blocked: bool) -> Result<(), ProbeError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err(ProbeError::Unavailable)
    }

    fn pci_devices(&self) -> Result<Vec<Device>, ProbeError> {
        Err(ProbeError::Unavailable)
    }

    fn usb_devices(&self) -> Result<Vec<Device>, ProbeError> {
        Err(ProbeError::Unavailable)
    }
}

#[test]
fn the_system_switches_through_rfkill() {
    let switches = Arc::new(Switches::default());
    let actions = SystemActions::new(Arc::<Switches>::clone(&switches));

    assert_eq!(
        actions.set_radio(RadioTarget::Wifi, true),
        Err(ProbeError::Unavailable)
    );
    assert_eq!(switches.0.load(Ordering::SeqCst), 1);
}
