//! The Boot options page's slow work: reading the boot menu, which may
//! ask for a password, and the file chooser, which waits for the user.
//! Each runs on a blocking thread and wakes the frame with its result.

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::runtime::Handle;
use toolbox_system::pkexec::HelperError;
use toolbox_system::{BootMenuSource, MenuRead};
pub use toolbox_system::{Chosen, FileChooser};

use crate::lane::{Asked, Lane};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    Background,
    Theme,
}

impl FileKind {
    /// The patterns of the files the dialog offers for this kind.
    pub const fn globs(self) -> &'static [&'static str] {
        match self {
            Self::Background => &["*.png", "*.jpg", "*.jpeg", "*.tga"],
            Self::Theme => &["*.txt"],
        }
    }
}

/// What a file dialog answered, with the kind of file it was asked for.
#[derive(Debug)]
pub enum Answer {
    Chose(Picked),
    Cancelled,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Picked {
    pub kind: FileKind,
    pub path: String,
}

pub struct BootSetup {
    pub runtime: Handle,
    /// Reads `grub.cfg`, through the helper where the user may not.
    pub menu: Arc<dyn BootMenuSource>,
    pub chooser: Arc<dyn FileChooser>,
}

#[derive(Debug)]
pub struct Jobs {
    setup: BootSetup,
    /// Set when the app ends, so a password dialog no longer holds a read.
    cancel: Arc<AtomicBool>,
    read: Asked<MenuRead>,
    picked: Lane<Answer>,
}

impl fmt::Debug for BootSetup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BootSetup").finish_non_exhaustive()
    }
}

impl Drop for Jobs {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

impl Jobs {
    pub fn new(setup: BootSetup, ctx: egui::Context) -> Self {
        let cancel: Arc<AtomicBool> = Arc::default();
        let read = Asked::new(
            setup.runtime.clone(),
            crate::waking::repaint(ctx.clone()),
            {
                let menu = Arc::clone(&setup.menu);
                let cancel = Arc::clone(&cancel);
                move |lane| {
                    let source = Arc::clone(&menu);
                    let cancel = Arc::clone(&cancel);
                    lane.blocking(
                        "menu",
                        move || source.menu(&cancel),
                        |read| read.unwrap_or(MenuRead::Unknown(HelperError::Stopped)),
                    );
                }
            },
        );

        Self {
            read,
            picked: Lane::new(setup.runtime.clone(), crate::waking::repaint(ctx)),
            setup,
            cancel,
        }
    }

    /// A chooser that panicked counts as a dialog that could not open.
    pub fn choose(&self, kind: FileKind, title: String, filter: String) {
        let chooser = Arc::clone(&self.setup.chooser);
        self.picked.blocking(
            "file",
            move || chooser.choose(&title, &filter, kind.globs()),
            move |chosen| match chosen {
                Ok(Chosen::File(path)) => Answer::Chose(Picked { kind, path }),
                Ok(Chosen::Cancelled) => Answer::Cancelled,
                Ok(Chosen::Failed) | Err(_) => Answer::Failed,
            },
        );
    }

    /// Starts a read of the menu. Returns whether one started.
    pub(super) fn read_menu(&mut self) -> bool {
        self.read.read()
    }

    pub const fn reading(&self) -> bool {
        self.read.running()
    }

    pub(super) fn menu_read(&mut self) -> Option<MenuRead> {
        self.read.take()
    }

    /// `None` while no dialog answered.
    pub fn picked(&self) -> Option<Answer> {
        self.picked.landed().pop()
    }
}
