#[cfg(test)]
mod tests;

use std::io;
use std::path::Path;

use toolbox_plan::KernelParameters;
use toolbox_plan::sdboot::KERNEL_CMDLINE;

use super::FileChange;
use crate::secure;

const BACKUP: &str = "cmdline.bak";

/// # Errors
///
/// When a folder on the way or the file is a symlink, the file is not
/// UTF-8, or the write fails.
pub fn write_kernel_cmdline(root: &Path, parameters: &KernelParameters) -> io::Result<FileChange> {
    let (dir, name) = secure::open_parent(root, KERNEL_CMDLINE)?;
    let current = secure::read_text(&dir, name)?;

    super::rewritten(
        &dir,
        name,
        current.as_deref().unwrap_or_default(),
        &format!("{parameters}\n"),
        current.is_some().then_some(BACKUP),
    )
}
