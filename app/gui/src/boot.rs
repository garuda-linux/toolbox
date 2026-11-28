//! What the Boot options page keeps between frames: the boot menu as last
//! read, the refused edit with its value, a kernel the Kernels page
//! proposed as the default, and the file chooser's answer.
//!
//! The menu may need the root helper, so it is read on a blocking thread
//! and only once the page was opened, never at start.

mod edits;
mod jobs;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use jobs::Jobs;
pub use jobs::{BootSetup, Chosen, FileChooser, FileKind, Picked};
use toolbox_gui_kit::row_cache::Same;
use toolbox_plan::PackageName;
use toolbox_services::boot_menu::Menu;
use toolbox_system::MenuRead;

/// An edit the page did not queue, with the value as the user gave it.
pub type BootRefusal = crate::refused::Refused<toolbox_core::features::boot::edit::EditCode>;

#[derive(Debug, Default)]
pub struct Boot {
    /// Shared, so the page tells a new menu from the one its entry list
    /// was built from without comparing them.
    menu: Arc<Menu>,
    /// `None` in tests that need no jobs: nothing is read or chosen.
    jobs: Option<Jobs>,
    refusal: Option<BootRefusal>,
    proposal: Option<PackageName>,
    /// A file dialog is open. A second one would drop the first's answer.
    choosing: bool,
    /// The last file dialog could not open, until the next one is asked
    /// for.
    chooser_failed: bool,
}

impl Boot {
    pub fn new(setup: Option<BootSetup>, ctx: egui::Context) -> Self {
        Self {
            jobs: setup.map(|setup| Jobs::new(setup, ctx)),
            ..Self::default()
        }
    }

    pub fn menu(&self) -> &Menu {
        &self.menu
    }

    /// The menu as last read, as an identity a row cache keys on.
    pub fn menu_identity(&self) -> Same<Menu> {
        Same::of(&self.menu)
    }

    pub fn open(&mut self, readable: bool) {
        if readable && *self.menu == Menu::NotAsked {
            self.read();
        }
    }

    /// Reads the menu again, if the page ever asked for it.
    pub fn refresh(&mut self) {
        if *self.menu != Menu::NotAsked {
            self.read();
        }
    }

    /// A plan may have generated the menu again.
    pub fn plan_finished(&mut self) {
        self.refresh();
    }

    /// Asks the desktop for a file of `kind`, unless its dialog is open
    /// already. Its answer comes from [`Self::take`].
    pub fn choose(&mut self, kind: FileKind, title: &str, filter: &str) {
        if self.choosing {
            return;
        }

        if let Some(jobs) = &self.jobs {
            self.choosing = true;
            self.chooser_failed = false;
            jobs.choose(kind, title.to_owned(), filter.to_owned());
        }
    }

    pub const fn choosing(&self) -> bool {
        self.choosing
    }

    pub const fn chooser_failed(&self) -> bool {
        self.chooser_failed
    }

    /// Takes the results that came since the last frame. Returns a file
    /// the user chose.
    pub fn take(&mut self) -> Option<Picked> {
        let jobs = self.jobs.as_mut()?;
        if let Some(read) = jobs.menu_read().filter(|read| *read != MenuRead::Cancelled) {
            self.menu = Arc::new(Menu::Read(read));
        }

        let answered = jobs.picked()?;
        self.choosing = false;
        match answered {
            jobs::Answer::Chose(picked) => Some(picked),
            jobs::Answer::Cancelled => None,
            jobs::Answer::Failed => {
                self.chooser_failed = true;
                None
            }
        }
    }

    /// Whether the menu is read, for a Read again that spins meanwhile.
    pub fn reading(&self) -> bool {
        self.jobs.as_ref().is_some_and(Jobs::reading)
    }

    pub const fn refusal(&self) -> Option<&BootRefusal> {
        self.refusal.as_ref()
    }

    pub fn set_refusal(&mut self, refusal: Option<BootRefusal>) {
        self.refusal = refusal;
    }

    pub const fn proposal(&self) -> Option<&PackageName> {
        self.proposal.as_ref()
    }

    pub fn propose(&mut self, kernel: Option<PackageName>) {
        self.proposal = kernel;
    }

    fn read(&mut self) {
        let Some(jobs) = &mut self.jobs else {
            return;
        };

        if !jobs.read_menu() {
            return;
        }

        // The last menu stays on screen while it is read again.
        if !matches!(*self.menu, Menu::Read(_)) {
            self.menu = Arc::new(Menu::Loading);
        }
    }
}

#[cfg(any(test, feature = "testing"))]
impl Boot {
    pub fn showing(
        menu: Menu,
        refusal: Option<BootRefusal>,
        proposal: Option<PackageName>,
    ) -> Self {
        Self {
            menu: Arc::new(menu),
            refusal,
            proposal,
            ..Self::default()
        }
    }

    #[must_use = "returns the new value, the old one stays as it was"]
    pub const fn with_failed_chooser(mut self) -> Self {
        self.chooser_failed = true;
        self
    }
}
