//! What the Boot options page keeps between frames: the boot menu as last
//! read, the refused edit with its value, a kernel the Kernels page
//! proposed as the default, and the file chooser's answer.
//!
//! The menu may need the root helper, so it is read on a blocking thread
//! and only once the page was opened, never at start.

mod jobs;
pub mod portal;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use jobs::Jobs;
pub use jobs::{BootSetup, FileChooser, FileKind, Picked};
use toolbox_gui_kit::row_cache::Same;
use toolbox_plan::PackageName;
use toolbox_system::{Menu, MenuRead};

use crate::lane::OneRead;

/// An edit the page did not queue: the code of why, and the value as the
/// user gave it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BootRefusal {
    pub code: &'static str,
    pub value: String,
}

#[derive(Debug, Default)]
pub struct Boot {
    /// Shared, so the page tells a new menu from the one its entry list
    /// was built from without comparing them.
    menu: Arc<Menu>,
    /// `None` in tests that need no jobs: nothing is read or chosen.
    jobs: Option<Jobs>,
    reads: OneRead,
    refusal: Option<BootRefusal>,
    proposal: Option<PackageName>,
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

    /// Asks the desktop for a file of `kind`. Its answer comes from
    /// [`Self::take`].
    pub fn choose(&self, kind: FileKind, title: &str, filter: &str) {
        if let Some(jobs) = &self.jobs {
            jobs.choose(kind, title.to_owned(), filter.to_owned());
        }
    }

    /// Takes the results that came since the last frame. Returns a file
    /// the user chose.
    pub fn take(&mut self) -> Option<Picked> {
        let jobs = self.jobs.as_ref()?;
        if let Some(read) = jobs.menu_read() {
            if read != MenuRead::Cancelled {
                self.menu = Arc::new(Menu::Read(read));
            }

            if self.reads.landed() {
                self.read();
            }
        }

        self.jobs.as_ref()?.picked()
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
        let Some(jobs) = &self.jobs else {
            return;
        };

        if !self.reads.start() {
            return;
        }

        // The last menu stays on screen while it is read again.
        if !matches!(*self.menu, Menu::Read(_)) {
            self.menu = Arc::new(Menu::Loading);
        }

        jobs.read_menu();
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
}
