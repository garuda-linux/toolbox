//! The Kernels page's clicks, the last one it refused, and the dkms
//! reading that the build changes and the page's rows need.

#[cfg(test)]
mod tests;

use toolbox_core::features::kernels::view::{BootMenus, Listing};
use toolbox_core::features::kernels::{
    self as core_kernels, DkmsEntry, install_missing_headers, set_builds, set_headers,
};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui_kit::Loadable;
use toolbox_plan::{PackageName, Toggle};
use toolbox_services::dkms::Dkms;

use crate::command::{AppCommand, KernelCommand, KernelEdit};
use crate::page_service::Handled;
use crate::reader::{ReaderHandle, Watched};
use crate::refused::{Refused, queue, queue_or_keep};

pub type KernelRefusal = Refused<core_kernels::Refusal, PackageName>;

/// What dkms said last. It is slow, so it is no part of the snapshot.
pub type DkmsReading = Loadable<Dkms>;

#[derive(Debug, Default)]
pub struct Kernels {
    refusal: Option<KernelRefusal>,
    /// `dkms status` runs on a blocking thread, at start and after plans.
    dkms: Watched<Dkms>,
}

impl Kernels {
    pub fn new(dkms: Option<ReaderHandle<Dkms>>) -> Self {
        Self {
            refusal: None,
            dkms: Watched::new(dkms),
        }
    }

    pub const fn refusal(&self) -> Option<&KernelRefusal> {
        self.refusal.as_ref()
    }

    pub fn set_refusal(&mut self, refusal: Option<KernelRefusal>) {
        self.refusal = refusal;
    }

    pub const fn dkms(&self) -> &DkmsReading {
        self.dkms.reading()
    }

    /// Whether dkms is asked, for a Read again that spins meanwhile.
    pub const fn reading(&self) -> bool {
        self.dkms.reading().is_loading()
    }

    #[cfg(feature = "testing")]
    pub const fn awaits(&self) -> bool {
        self.dkms.awaits()
    }

    /// The modules of the last answer, which stay while dkms is asked
    /// again. Empty until dkms answered.
    pub fn dkms_entries(&self) -> &[DkmsEntry] {
        self.dkms().get().map_or(&[], Dkms::entries)
    }

    /// Changes with every dkms answer, so the page knows its rows are old.
    pub const fn dkms_revision(&self) -> u64 {
        self.dkms.revision()
    }

    /// The reading a test hands the page, without a reader.
    #[cfg(any(test, feature = "testing"))]
    pub fn with_dkms(dkms: DkmsReading) -> Self {
        Self {
            dkms: Watched::loaded(dkms),
            ..Self::default()
        }
    }

    /// What `dkms status` printed since the last frame.
    pub fn take_dkms(&mut self) {
        self.dkms.take();
    }

    /// A plan may have installed a kernel or built a module.
    pub fn read_dkms(&self) {
        self.dkms.refresh();
    }

    pub fn handle(
        &mut self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        let AppCommand::Kernel(command) = command else {
            return Handled::default();
        };

        let edit = match command {
            KernelCommand::Edit(edit) => edit,
            KernelCommand::Refresh => {
                self.dkms.refresh();
                return Handled::answers(vec![AppCommand::ReadSystem]);
            }
        };

        let Some(listing) = Listing::of(snapshot, BootMenus::default()) else {
            return Handled::taken();
        };

        match edit {
            KernelEdit::Toggle { kernel, on } => {
                let Some(found) = listing.kernel(kernel) else {
                    return Handled::taken();
                };

                queue_or_keep(&mut self.refusal, changes, |changes| {
                    core_kernels::toggle(
                        found,
                        Toggle::from_on(*on),
                        listing.listed(),
                        changes,
                        snapshot,
                    )
                })
            }
            KernelEdit::Headers { kernel, on } => {
                listing.kernel(kernel).map_or_else(Handled::taken, |found| {
                    queue(changes, |changes| {
                        set_headers(found, Toggle::from_on(*on), changes, snapshot);
                    })
                })
            }
            KernelEdit::AllHeaders => queue(changes, |changes| {
                install_missing_headers(listing.kernels(), changes, snapshot);
            }),
            KernelEdit::Builds { kernel, on } => {
                listing.kernel(kernel).map_or_else(Handled::taken, |found| {
                    let builds = listing.builds(found, self.dkms_entries());
                    queue(changes, |changes| {
                        set_builds(&builds, Toggle::from_on(*on), changes, snapshot);
                    })
                })
            }
            KernelEdit::AllBuilds => {
                let builds = listing.all_builds(self.dkms_entries());
                queue(changes, |changes| {
                    set_builds(&builds, Toggle::On, changes, snapshot);
                })
            }
        }
    }
}
