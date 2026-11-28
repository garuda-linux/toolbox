//! The desktop's own file dialog through the XDG file chooser portal, so it
//! looks native on every desktop that has a portal.

use ashpd::Error;
use ashpd::desktop::ResponseError;
use ashpd::desktop::file_chooser::{FileFilter, SelectedFiles};
use tokio::runtime::Handle;

use crate::file_path;

/// What the desktop's file dialog gave back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Chosen {
    /// The chosen file's path.
    File(String),
    /// The user closed the dialog.
    Cancelled,
    /// No dialog could open, as on a desktop without a file chooser
    /// portal, or it gave back no local file.
    Failed,
}

/// Asks the desktop for one file. Blocks until the user answers.
pub trait FileChooser: Send + Sync {
    /// `title` names the dialog, `filter` the kind of file it lists and
    /// `globs` the patterns of the files it offers.
    fn choose(&self, title: &str, filter: &str, globs: &[&str]) -> Chosen;
}

/// The portal's chooser. Without a portal nothing opens and the answer is
/// a failure, which the page tells from a cancel.
#[derive(Debug)]
pub struct PortalChooser {
    runtime: Handle,
}

impl PortalChooser {
    pub const fn new(runtime: Handle) -> Self {
        Self { runtime }
    }
}

impl FileChooser for PortalChooser {
    fn choose(&self, title: &str, filter: &str, globs: &[&str]) -> Chosen {
        let filter = globs
            .iter()
            .fold(FileFilter::new(filter), |filter, glob| filter.glob(glob));

        // A blocking thread may wait for the portal's future.
        let answer = self.runtime.block_on(async {
            SelectedFiles::open_file()
                .title(title)
                .modal(true)
                .filter(filter)
                .send()
                .await?
                .response()
        });

        match answer {
            Ok(files) => {
                let path = files.uris().first().and_then(|uri| file_path(uri.as_str()));
                path.map_or_else(
                    || {
                        tracing::warn!("the file dialog gave back no local file");
                        Chosen::Failed
                    },
                    Chosen::File,
                )
            }
            Err(Error::Response(ResponseError::Cancelled)) => Chosen::Cancelled,
            Err(error) => {
                tracing::warn!("the file dialog could not open: {error}");
                Chosen::Failed
            }
        }
    }
}
