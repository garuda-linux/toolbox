//! What `queue add` reads of the running system beyond the snapshot, behind
//! one port so the parity test can answer from memory.

use tokio::runtime::Runtime;
use toolbox_cli::output::ColorMode;
use toolbox_plan::UnitScope;
use toolbox_plan::sdboot::SdBootMenu;
use toolbox_services::programs::Programs;
use toolbox_services::units::{Systemd, UnitList, UnitLister as _};
use toolbox_state::kernels::DkmsEntry;
use toolbox_system::{Dkms, DkmsSource as _, ProbeError};

pub trait SystemReads {
    /// The units of `scope`, for the lookup a service change starts from.
    fn units(&self, scope: UnitScope) -> UnitList;

    /// # Errors
    ///
    /// As [`toolbox_system::DkmsSource::status`].
    fn dkms(&self) -> Result<Vec<DkmsEntry>, ProbeError>;

    /// Whether garuda-update is installed, which the upgrade prefers.
    fn garuda_update(&self) -> bool;

    /// The systemd-boot menu, which a loader setting is checked against.
    fn boot_menu(&self) -> Option<SdBootMenu>;
}

/// The real system: systemd over D-Bus, `dkms status`, the programs on
/// disk and the boot menu through the helper.
pub struct LiveReads<'a> {
    pub runtime: &'a Runtime,
    /// For the helper's password prompt.
    pub color: ColorMode,
}

impl SystemReads for LiveReads<'_> {
    fn units(&self, scope: UnitScope) -> UnitList {
        self.runtime.block_on(Systemd.list(scope))
    }

    fn dkms(&self) -> Result<Vec<DkmsEntry>, ProbeError> {
        Dkms::system().status()
    }

    fn garuda_update(&self) -> bool {
        Programs::check().garuda_update
    }

    fn boot_menu(&self) -> Option<SdBootMenu> {
        crate::boot::sdboot::read_menu(self.color)
    }
}
