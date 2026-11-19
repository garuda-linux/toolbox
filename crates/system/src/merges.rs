//! Both files of a copy pacman left, read as the user where the user may,
//! else through the helper, as the Config Files page reads.

#[cfg(test)]
mod tests;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use toolbox_plan::{ConfigText, FetchRefusal, MergeCopy};

use crate::config_files::{ConfigFileSource, ReadProblem};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MergePair {
    pub copy: MergeCopy,
    pub copied: Result<ConfigText, ReadProblem>,
    /// `None` when the file in use is gone.
    pub current: Result<Option<ConfigText>, ReadProblem>,
}

/// Blocks, and may ask for a password once per file only root may read.
pub fn read_pair(
    source: &dyn ConfigFileSource,
    copy: &MergeCopy,
    cancel: &Arc<AtomicBool>,
) -> MergePair {
    let copied = source.read(copy.path(), cancel).map(|read| read.content);
    let current = match source.read(copy.original(), cancel) {
        Ok(read) => Ok(Some(read.content)),
        Err(ReadProblem::Refused(FetchRefusal::Missing)) => Ok(None),
        Err(problem) => Err(problem),
    };

    MergePair {
        copy: copy.clone(),
        copied,
        current,
    }
}
