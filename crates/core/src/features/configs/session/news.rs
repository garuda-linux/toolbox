//! What goes into an edit and comes out of it: the work a step asks
//! for and the results the worker threads bring back.

use toolbox_plan::{ConfigText, EditCopy, Editor};

use super::file::{EditWrite, EditedFile};

/// What a step needs done outside the state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Work {
    /// Read the file, pick the editor and write the private copy.
    Fetch(EditedFile),
    RunEditor(Editor, EditCopy),
    /// Read the copy back and compare it with the original.
    Compare(EditCopy, ConfigText),
    Write(EditWrite),
    /// Read the file again and compare the edited text with it.
    Reread(EditedFile, ConfigText),
    RemoveCopy(EditCopy),
    /// Stop the running editor.
    Cancel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fetched {
    pub editor: Editor,
    pub copy: EditCopy,
    pub original: ConfigText,
    /// The file the write goes to: for a root-owned file the real path
    /// a link leads to, which the helper checks again. A file of the user
    /// keeps its path, and the write follows the link itself.
    pub target: EditedFile,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Compared {
    Unchanged,
    Changed(Changes),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Changes {
    /// The file as it is now, which the write must still find.
    pub original: ConfigText,
    pub edited: ConfigText,
    /// For the screen reader and for themes without colours: `+` and `-`
    /// in front of the lines.
    pub plain: String,
    /// The same with terminal colour codes.
    pub colored: String,
    pub added: usize,
    pub removed: usize,
}

/// Why a step failed: a code the pages have words for, and the system's own
/// words when it has some.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Problem {
    pub code: &'static str,
    pub detail: String,
}

impl Problem {
    pub fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}
