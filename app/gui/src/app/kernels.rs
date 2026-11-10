//! What the app does with a click on the Kernels page.

use toolbox_core::features::kernels::{install_missing_headers, toggle};
use toolbox_plan::Toggle;

use super::App;
use crate::command::{KernelCommand, KernelEdit};
use crate::pages::KernelRefusal;
use crate::pages::kernels::view::Listing;
use crate::pages::kernels::{set_builds, set_headers};

impl App {
    pub(super) fn run_kernel(&mut self, command: KernelCommand) {
        match command {
            KernelCommand::Refresh => {
                self.read_system();
                self.dkms.refresh();
            }
            KernelCommand::DismissRefusal => self.refusals.kernel = None,
            KernelCommand::Edit(edit) => self.edit_kernels(edit),
        }
    }

    /// Changes the queue against the current snapshot.
    fn edit_kernels(&mut self, edit: KernelEdit) {
        let snapshot = self.current_snapshot();
        let Some(listing) = Listing::of(&snapshot) else {
            return;
        };

        // Its own handle, as the queue borrows the app mutably below.
        let dkms = self.dkms.reading().clone();

        match edit {
            KernelEdit::Toggle { kernel, on } => {
                let Some(found) = listing.kernel(&kernel) else {
                    return;
                };

                let result = self.queue.edit(&snapshot, |changes, snapshot| {
                    toggle(
                        found,
                        Toggle::from_on(on),
                        listing.listed(),
                        changes,
                        snapshot,
                    )
                });

                self.refusals.kernel = result
                    .err()
                    .map(|refusal| KernelRefusal { refusal, kernel });
            }
            KernelEdit::Headers { kernel, on } => {
                if let Some(found) = listing.kernel(&kernel) {
                    self.queue.edit(&snapshot, |changes, snapshot| {
                        set_headers(found, Toggle::from_on(on), changes, snapshot)
                    });
                }
            }
            KernelEdit::Builds { kernel, on } => {
                if let Some(found) = listing.kernel(&kernel) {
                    self.queue.edit(&snapshot, |changes, snapshot| {
                        set_builds(
                            &listing.builds(found, &dkms),
                            Toggle::from_on(on),
                            changes,
                            snapshot,
                        );
                    });
                }
            }
            KernelEdit::AllHeaders => {
                self.queue.edit(&snapshot, |changes, snapshot| {
                    install_missing_headers(listing.kernels(), changes, snapshot)
                });
            }
            KernelEdit::AllBuilds => {
                self.queue.edit(&snapshot, |changes, snapshot| {
                    set_builds(&listing.all_builds(&dkms), Toggle::On, changes, snapshot);
                });
            }
        }
    }
}
