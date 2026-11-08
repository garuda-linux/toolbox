//! Starting the app at login: a link in the XDG autostart folder to the
//! installed desktop file. The link itself is the setting, so it can never
//! disagree with a saved flag.

#[cfg(test)]
mod tests;

use std::io;
use std::path::{Path, PathBuf};

pub const DESKTOP_FILE: &str = "/usr/share/applications/garuda-toolbox.desktop";

const LINK_NAME: &str = "garuda-toolbox.desktop";

/// Entries of the app under its former name Garuda Rani.
const LEGACY_NAMES: [&str; 2] = ["garuda-rani.desktop", "org.garudalinux.rani.desktop"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Autostart {
    folder: PathBuf,
    desktop_file: PathBuf,
}

impl Autostart {
    #[must_use = "an autostart handle does nothing until used"]
    pub const fn new(folder: PathBuf, desktop_file: PathBuf) -> Self {
        Self {
            folder,
            desktop_file,
        }
    }

    /// A link counts even when it dangles: the desktop starts nothing then,
    /// but the user still sees it as on and can turn it off.
    pub fn is_enabled(&self) -> bool {
        self.link().symlink_metadata().is_ok()
    }

    /// Also removes the entries of the former name, which would start the
    /// app a second time.
    ///
    /// # Errors
    ///
    /// When the folder or the link cannot be created, or an old entry
    /// cannot be removed.
    pub fn enable(&self) -> io::Result<()> {
        std::fs::create_dir_all(&self.folder)?;
        remove_if_present(&self.link())?;
        self.remove_legacy()?;
        std::os::unix::fs::symlink(&self.desktop_file, self.link())?;
        tracing::info!("autostart on: {}", self.link().display());
        Ok(())
    }

    /// Also removes the entries of the former name, which would start the
    /// app a second time.
    ///
    /// # Errors
    ///
    /// When an existing entry cannot be removed.
    pub fn disable(&self) -> io::Result<()> {
        remove_if_present(&self.link())?;
        self.remove_legacy()?;
        tracing::info!("autostart off");
        Ok(())
    }

    fn remove_legacy(&self) -> io::Result<()> {
        for name in LEGACY_NAMES {
            remove_if_present(&self.folder.join(name))?;
        }

        Ok(())
    }

    fn link(&self) -> PathBuf {
        self.folder.join(LINK_NAME)
    }
}

fn remove_if_present(path: &Path) -> io::Result<()> {
    match std::fs::remove_file(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}
