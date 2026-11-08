//! A root-owned file is read as the user when the user may read it, with
//! the code the helper uses, so both refuse the same files. Only when the
//! system denies the read does the helper hand the text out, after an
//! administrator's password.

mod copies;
mod editing;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::{fs, io};

pub use copies::EditFolder;
pub use editing::{DryRunEditor, Editing, SystemEditor};
use toolbox_exec::{ConfigRead, MAX_FOLDER_FILES, ReadError, inspect, read_config_file};
use toolbox_plan::protocol::MAX_LINE_BYTES;
use toolbox_plan::{ConfigFilePath, Editor, FetchAnswer, FetchRefusal, HomePath};
use toolbox_state::configs::{ConfigEntryLocation, ConfigEntrySpec, Found, Link, Listing};

use crate::pkexec::{self, HelperRun};

pub trait HelperFetch: Send + Sync {
    /// The command that prints one [`FetchAnswer`] line for `path`.
    fn fetch_command(&self, path: &ConfigFilePath) -> Command;
}

impl<H> HelperFetch for Arc<H>
where
    H: HelperFetch + ?Sized,
{
    fn fetch_command(&self, path: &ConfigFilePath) -> Command {
        H::fetch_command(self, path)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadProblem {
    /// The file is there for nobody to edit through the toolbox.
    Refused(FetchRefusal),
    NotRun(pkexec::Refusal),
    Failed(String),
    Cancelled,
}

/// Every call blocks.
pub trait ConfigFileSource: Send + Sync {
    fn listing(&self, entries: &[ConfigEntrySpec]) -> Listing;

    /// The real path it was read at, where its symlinks lead.
    ///
    /// # Errors
    ///
    /// [`ReadProblem`] when the file is refused, the user is not
    /// authorized, the read fails or `cancel` ended it.
    fn read(
        &self,
        path: &ConfigFilePath,
        cancel: &Arc<AtomicBool>,
    ) -> Result<ConfigRead, ReadProblem>;

    fn installed(&self, editor: Editor) -> bool;

    fn home_file(&self, path: &HomePath) -> Option<PathBuf>;
}

pub struct SystemConfigFiles<H> {
    root: PathBuf,
    home: Option<PathBuf>,
    helper: H,
}

impl<H> SystemConfigFiles<H> {
    #[must_use = "the adapter does nothing until asked"]
    pub fn system(helper: H) -> Self {
        Self {
            root: PathBuf::from("/"),
            home: std::env::var_os("HOME")
                .filter(|home| !home.is_empty())
                .map(PathBuf::from),
            helper,
        }
    }

    /// Reads below `root` and `home`, for tests.
    #[must_use = "the adapter does nothing until asked"]
    pub fn new(root: impl Into<PathBuf>, home: impl Into<PathBuf>, helper: H) -> Self {
        Self {
            root: root.into(),
            home: Some(home.into()),
            helper,
        }
    }

    fn found(&self, entry: &ConfigEntrySpec) -> Option<Found> {
        match &entry.location {
            ConfigEntryLocation::Home(path) => home_entry(
                &self.home.as_ref()?.join(path.as_str()),
                path,
                entry.directory,
            ),
            ConfigEntryLocation::Root(path) => root_entry(&self.root, path, entry.directory),
        }
    }
}

impl<H> ConfigFileSource for SystemConfigFiles<H>
where
    H: HelperFetch,
{
    fn listing(&self, entries: &[ConfigEntrySpec]) -> Listing {
        let mut listing = Listing::default();

        for entry in entries {
            if let Some(found) = self.found(entry) {
                listing.insert(entry.id.clone(), found);
            }
        }

        listing
    }

    fn read(
        &self,
        path: &ConfigFilePath,
        cancel: &Arc<AtomicBool>,
    ) -> Result<ConfigRead, ReadProblem> {
        match read_config_file(&self.root, path) {
            Ok(read) => Ok(read),
            Err(ReadError::Refused(reason)) => Err(ReadProblem::Refused(reason)),
            Err(ReadError::Failed(error)) if error.kind() == io::ErrorKind::PermissionDenied => {
                tracing::info!("{path} is kept from the user, asking the helper");
                fetch(self.helper.fetch_command(path), cancel)
            }
            Err(ReadError::Failed(error)) => Err(ReadProblem::Failed(error.to_string())),
        }
    }

    fn installed(&self, editor: Editor) -> bool {
        // The real system also finds an editor in the user's PATH, as the
        // Console runs it. A test root has only its own files.
        if self.root == Path::new("/") {
            return toolbox_exec::locate_editor(editor.program()).is_some();
        }

        editor.program().below(&self.root).is_file()
    }

    fn home_file(&self, path: &HomePath) -> Option<PathBuf> {
        self.home.as_ref().map(|home| home.join(path.as_str()))
    }
}

/// Symlinks are followed: dotfile managers link the files in `~/.config`, and the user's own rights are the only limit.
fn home_entry(path: &Path, relative: &HomePath, directory: bool) -> Option<Found> {
    let metadata = fs::metadata(path).ok()?;

    if metadata.is_file() {
        return Some(Found::File { link: None });
    }

    if !metadata.is_dir() || !directory {
        return None;
    }

    let mut names: Vec<String> = fs::read_dir(path)
        .ok()?
        .flatten()
        .filter(|file| file.path().is_file())
        .filter_map(|file| file.file_name().into_string().ok())
        .filter(|name| HomePath::new(format!("{}/{name}", relative.as_str())).is_ok())
        .collect();

    names.sort();
    names.truncate(MAX_FOLDER_FILES);
    Some(Found::Folder(names))
}

/// What the user cannot even see (a folder on the way that only root may enter)
/// counts as missing: the page shows existing files only, and existence cannot be told.
/// A symlink is shown with where it leads, as the helper would follow it: the same code looks.
fn root_entry(root: &Path, path: &ConfigFilePath, directory: bool) -> Option<Found> {
    match inspect(root, path) {
        toolbox_exec::Found::File { link } => Some(Found::File {
            link: link.as_ref().map(Link::to),
        }),
        toolbox_exec::Found::Link { target, reason } => Some(Found::File {
            link: Some(Link::refused(target, reason)),
        }),
        toolbox_exec::Found::Folder { files, link: _ } if directory => Some(Found::Folder(
            files
                .into_iter()
                .filter(|name| path.join(name).is_ok())
                .collect(),
        )),
        // A link where the catalog has a file is worth a line that says
        // why it cannot be edited. A plain folder there is just no file.
        toolbox_exec::Found::Folder {
            files: _,
            link: Some(target),
        } => Some(Found::File {
            link: Some(Link::refused(target.to_string(), FetchRefusal::NotRegular)),
        }),
        toolbox_exec::Found::Folder {
            files: _,
            link: None,
        }
        | toolbox_exec::Found::Missing
        | toolbox_exec::Found::Hidden
        | toolbox_exec::Found::Other => None,
    }
}

/// The content is validated while decoding, so anything else is refused whole.
fn fetch(mut command: Command, cancel: &Arc<AtomicBool>) -> Result<ConfigRead, ReadProblem> {
    let line = match pkexec::ask(&mut command, MAX_LINE_BYTES, cancel) {
        HelperRun::Answered(line) => line,
        HelperRun::Refused(refusal) => return Err(ReadProblem::NotRun(refusal)),
        HelperRun::Failed(reason) => return Err(ReadProblem::Failed(reason)),
        HelperRun::Cancelled => return Err(ReadProblem::Cancelled),
    };

    let answer = FetchAnswer::parse(line.trim_end()).map_err(|_not_a_file| {
        ReadProblem::Failed("the helper's answer is no config file".to_owned())
    })?;

    match answer {
        FetchAnswer::Text { path, content } => Ok(ConfigRead { path, content }),
        FetchAnswer::Refused { reason } => Err(ReadProblem::Refused(reason)),
    }
}
