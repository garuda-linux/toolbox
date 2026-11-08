//! How the editor of a config file edit plays its part. The system
//! starts the editor the user picked. A dry run starts no program at all,
//! so its stand-in needs none installed and makes the change itself.

#[cfg(test)]
mod tests;

use std::io;

use toolbox_plan::{EditCopy, Editor};

use super::EditFolder;

pub trait Editing: Send + Sync {
    /// The editor to start, given the one `$VISUAL` or `$EDITOR` picked
    /// among the installed ones. `None` refuses the edit.
    fn editor(&self, chosen: Option<Editor>) -> Option<Editor>;

    /// Runs once the editor step ended, before the copy is read back.
    ///
    /// # Errors
    ///
    /// When the copy cannot be written.
    fn after_edit(&self, copies: &EditFolder, copy: &EditCopy) -> io::Result<()>;
}

pub struct SystemEditor;

impl Editing for SystemEditor {
    fn editor(&self, chosen: Option<Editor>) -> Option<Editor> {
        chosen
    }

    fn after_edit(&self, _copies: &EditFolder, _copy: &EditCopy) -> io::Result<()> {
        Ok(())
    }
}

/// No editor runs, so one comment line added here gives the steps a
/// change to review.
pub struct DryRunEditor;

impl Editing for DryRunEditor {
    fn editor(&self, chosen: Option<Editor>) -> Option<Editor> {
        chosen.or(Some(Editor::Micro))
    }

    fn after_edit(&self, copies: &EditFolder, copy: &EditCopy) -> io::Result<()> {
        copies.stand_in_edit(copy)
    }
}
