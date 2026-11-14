//! What goes into an edit and comes out of it: the work a step asks
//! for and the results the worker threads bring back.

use toolbox_plan::{ConfigText, EditCopy, Editor, FetchRefusal};

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    pub code: ProblemCode,
    pub detail: String,
}

/// The password dialog's answers mirror `pkexec::Refusal` and the helper's
/// refusals mirror [`FetchRefusal`], since each has a sentence of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum ProblemCode {
    Dismissed,
    Denied,
    NoAgent,
    HelperMissing,
    /// pkexec cannot work on this system.
    Broken,
    Missing,
    Link,
    Dangling,
    Outside,
    NotRegular,
    TooLarge,
    NotText,
    NoEditor,
    /// Another task holds the Console.
    Busy,
    EditorFailed,
    WriteFailed,
    CopyFailed,
    ReadFailed,
    /// A file of the user, on a system that names no home folder.
    NoHome,
    /// The app closed while the read waited, maybe on a password dialog.
    Cancelled,
    /// The worker thread of a step stopped without an answer.
    Failed,
}

impl From<FetchRefusal> for ProblemCode {
    fn from(reason: FetchRefusal) -> Self {
        match reason {
            FetchRefusal::Missing => Self::Missing,
            FetchRefusal::Link => Self::Link,
            FetchRefusal::Dangling => Self::Dangling,
            FetchRefusal::Outside => Self::Outside,
            FetchRefusal::NotRegular => Self::NotRegular,
            FetchRefusal::TooLarge => Self::TooLarge,
            FetchRefusal::NotText => Self::NotText,
        }
    }
}

impl Problem {
    pub fn new(code: ProblemCode, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}
