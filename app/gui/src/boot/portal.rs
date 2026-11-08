//! The desktop's own file dialog through the XDG file chooser portal, so
//! it looks native on every desktop that has a portal.

use ashpd::desktop::file_chooser::{FileFilter, SelectedFiles};
use tokio::runtime::Handle;
use toolbox_system::file_path;

use super::{FileChooser, FileKind};

/// The portal's chooser. Without a portal nothing opens and the answer is
/// no file.
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
    fn choose(&self, kind: FileKind, title: &str, filter: &str) -> Option<String> {
        let globs: &[&str] = match kind {
            FileKind::Background => &["*.png", "*.jpg", "*.jpeg", "*.tga"],
            FileKind::Theme => &["*.txt"],
        };

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
            Ok(files) => files.uris().first().and_then(|uri| file_path(uri.as_str())),
            Err(error) => {
                tracing::info!("no file was chosen: {error}");
                None
            }
        }
    }
}
