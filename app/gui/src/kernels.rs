//! The Kernels page's clicks, the last one it refused, and the dkms
//! reading that the build changes and the page's rows need.

#[cfg(test)]
mod tests;

use toolbox_core::features::kernels::{
    self as core_kernels, install_missing_headers, set_builds, set_headers,
};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::{PackageName, Toggle};
use toolbox_services::queue::edit::QueueEdit;

use crate::command::{AppCommand, KernelCommand, KernelEdit};
use crate::page_service::Handled;
use crate::pages::kernels::view::{Dkms, DkmsReading, Listing};
use crate::reader::{ReaderHandle, Watched};
use crate::refused::{Refused, queue_or_refuse};

pub type KernelRefusal = Refused<core_kernels::Refusal, PackageName>;

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
            KernelCommand::DismissRefusal => {
                self.refusal = None;
                return Handled::taken();
            }
            KernelCommand::Edit(edit) => edit,
            KernelCommand::Refresh => {
                self.dkms.refresh();
                return Handled::answers(vec![AppCommand::ReadSystem]);
            }
        };

        let Some(listing) = Listing::of(snapshot) else {
            return Handled::taken();
        };

        match edit {
            KernelEdit::Toggle { kernel, on } => {
                let Some(found) = listing.kernel(kernel) else {
                    return Handled::taken();
                };

                queue_or_refuse(&mut self.refusal, kernel.clone(), changes, |changes| {
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
                    click(changes, |changes| {
                        set_headers(found, Toggle::from_on(*on), changes, snapshot);
                    })
                })
            }
            KernelEdit::AllHeaders => click(changes, |changes| {
                install_missing_headers(listing.kernels(), changes, snapshot);
            }),
            KernelEdit::Builds { kernel, on } => {
                listing.kernel(kernel).map_or_else(Handled::taken, |found| {
                    let builds = listing.builds(found, self.dkms.reading());
                    click(changes, |changes| {
                        set_builds(&builds, Toggle::from_on(*on), changes, snapshot);
                    })
                })
            }
            KernelEdit::AllBuilds => {
                let builds = listing.all_builds(self.dkms.reading());
                click(changes, |changes| {
                    set_builds(&builds, Toggle::On, changes, snapshot);
                })
            }
        }
    }
}

fn click(changes: &ChangeSet, click: impl FnOnce(&mut ChangeSet)) -> Handled {
    Handled::edit(QueueEdit::of_click(changes, click).0)
}
