//! The Boot options page's slow work: reading the boot menu, which may
//! ask for a password, and the file chooser, which waits for the user.
//! Each runs on a blocking thread and wakes the frame with its result.

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::runtime::Handle;
use toolbox_system::{BootMenuSource, MenuRead};

use crate::lane::Lane;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    Background,
    Theme,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Picked {
    pub kind: FileKind,
    pub path: String,
}

/// Asks the desktop for one file. Blocks until the user answers.
pub trait FileChooser: Send + Sync {
    /// The chosen file's path, `None` when the user cancelled or no
    /// chooser could open. `title` names the dialog, `filter` the kind
    /// of file it lists.
    fn choose(&self, kind: FileKind, title: &str, filter: &str) -> Option<String>;
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
    read: Lane<MenuRead>,
    picked: Lane<Option<Picked>>,
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
        Self {
            read: Lane::new(setup.runtime.clone(), crate::waking::repaint(ctx.clone())),
            picked: Lane::new(setup.runtime.clone(), crate::waking::repaint(ctx)),
            setup,
            cancel: Arc::default(),
        }
    }

    /// A read that panicked counts as a menu that could not be read.
    pub fn read_menu(&self) {
        let source = Arc::clone(&self.setup.menu);
        let cancel = Arc::clone(&self.cancel);
        self.read.blocking(
            "menu",
            move || source.menu(&cancel),
            |read| read.unwrap_or_else(|stopped| MenuRead::Unknown(stopped.to_string())),
        );
    }

    pub fn choose(&self, kind: FileKind, title: String, filter: String) {
        let chooser = Arc::clone(&self.setup.chooser);
        self.picked.blocking(
            "file",
            move || chooser.choose(kind, &title, &filter),
            move |path| {
                Some(Picked {
                    kind,
                    path: path.ok().flatten()?,
                })
            },
        );
    }

    pub fn menu_read(&self) -> Option<MenuRead> {
        self.read.landed().pop()
    }

    pub fn picked(&self) -> Option<Picked> {
        self.picked.landed().into_iter().flatten().last()
    }
}
