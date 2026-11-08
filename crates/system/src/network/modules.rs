//! Kernel modules from files: `/proc/modules` for what is loaded,
//! `modules.dep` of the running kernel for what can be loaded, and the
//! modprobe folders for what is blacklisted.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use toolbox_plan::{KernelRelease, MODPROBE_BLACKLIST, blacklist_entry};
use toolbox_state::network::{BlacklistEntry, ModuleFacts, loaded_modules, network_modules};

use crate::error::{ProbeError, from_io};
use crate::files;
use crate::kernels::{MODULES, OS_RELEASE};

const PROC_MODULES: &str = "proc/modules";
const MODULES_DEP: &str = "modules.dep";

/// Where modprobe reads its configuration, first match first.
const MODPROBE_FOLDERS: [&str; 3] = ["etc/modprobe.d", "run/modprobe.d", "usr/lib/modprobe.d"];
const CONF: &str = "conf";

pub trait ModuleSource: Send + Sync {
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without `/proc/modules`,
    /// [`ProbeError::Failed`] when a file that exists cannot be read.
    fn facts(&self) -> Result<ModuleFacts, ProbeError>;
}

/// [`ModuleSource`] from the files below a root folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemModules {
    root: PathBuf,
}

impl SystemModules {
    #[must_use = "the adapter does nothing until asked"]
    pub fn system() -> Self {
        Self::new("/")
    }

    #[must_use = "the adapter does nothing until asked"]
    pub(crate) fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// `modules.dep` of the running kernel. `None` once an update removed
    /// that kernel's folder: until the restart nothing can be loaded
    /// anyway.
    fn modules_dep(&self) -> Result<Option<String>, ProbeError> {
        let release = files::read(&self.root.join(OS_RELEASE))?;
        let Ok(release) = KernelRelease::new(release.trim()) else {
            return Ok(None);
        };

        files::read_optional(
            &self
                .root
                .join(MODULES)
                .join(release.as_str())
                .join(MODULES_DEP),
        )
    }

    fn blacklists(&self) -> Vec<BlacklistEntry> {
        let mut seen = BTreeSet::new();

        MODPROBE_FOLDERS
            .iter()
            .flat_map(|folder| conf_files(&self.root.join(folder)))
            // modprobe reads one file per name, from the first folder.
            .filter(|file| seen.insert(file.file_name().map(ToOwned::to_owned)))
            .flat_map(|file| {
                let shown = self.shown_path(&file);
                let own = shown.trim_start_matches('/') == MODPROBE_BLACKLIST;
                let text = files::read_lossy(&file)
                    .inspect_err(|error| tracing::debug!("skipped {}: {error}", file.display()))
                    .unwrap_or_default();

                text.lines()
                    .filter_map(blacklist_entry)
                    .map(|module| BlacklistEntry {
                        module,
                        file: shown.clone(),
                        own,
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn shown_path(&self, file: &Path) -> String {
        let relative = file.strip_prefix(&self.root).unwrap_or(file);

        format!("/{}", relative.display())
    }
}

impl ModuleSource for SystemModules {
    fn facts(&self) -> Result<ModuleFacts, ProbeError> {
        let loaded = files::read(&self.root.join(PROC_MODULES))?;
        let modules_dep = self.modules_dep()?;

        Ok(ModuleFacts {
            loaded: loaded_modules(&loaded),
            modules: modules_dep
                .as_deref()
                .map(network_modules)
                .unwrap_or_default(),
            blacklists: self.blacklists(),
        })
    }
}

/// A missing folder has none.
fn conf_files(folder: &Path) -> Vec<PathBuf> {
    let entries = match std::fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(error) => {
            if error.kind() != std::io::ErrorKind::NotFound {
                tracing::debug!("{}", from_io(folder.display(), &error));
            }

            return Vec::new();
        }
    };

    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == CONF))
        .collect();

    files.sort();
    files
}
