use std::io;
use std::path::PathBuf;

use toolbox_plan::{InvalidConfigWord, InvalidHomePath, InvalidName, InvalidRgb};

#[cfg(test)]
mod tests;

/// Every variant names the file, so the log line points at what to fix.
#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("{}: {error}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        error: io::Error,
    },
    #[error("{source_name}: {message}")]
    Syntax {
        source_name: String,
        /// The parser's message with line and column.
        message: String,
    },
    #[error("{source_name}: entry {id:?}: {problem}")]
    Invalid {
        source_name: String,
        id: String,
        problem: Problem,
    },
}

impl CatalogError {
    /// The kind of failure, which the pages put in their own words.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Read { path: _, error: _ } => "unreadable",
            Self::Syntax {
                source_name: _,
                message: _,
            } => "syntax",
            Self::Invalid {
                source_name: _,
                id: _,
                problem: _,
            } => "invalid",
        }
    }
}

/// The detail is shown apart as plain text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadProblem {
    pub code: &'static str,
    pub detail: String,
}

impl From<&CatalogError> for LoadProblem {
    fn from(error: &CatalogError) -> Self {
        Self {
            code: error.code(),
            detail: error.to_string(),
        }
    }
}

impl LoadProblem {
    /// The first one's code, every message on its own line, `None` when all loaded.
    pub fn joined(problems: Vec<Self>) -> Option<Self> {
        let code = problems.first()?.code;
        let detail = problems
            .into_iter()
            .map(|problem| problem.detail)
            .collect::<Vec<_>>()
            .join("\n");

        Some(Self { code, detail })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Problem {
    #[error("the id must be lowercase words joined by dashes")]
    BadId,
    #[error("the id is used twice")]
    DuplicateId,
    /// Another entry of the same catalog watches the same state, so one
    /// switch would flip the other.
    #[error("another entry watches {0:?}")]
    DuplicateCheck(String),
    /// Another config file entry points at the same path, so an edit in
    /// one would show in the other.
    #[error("another entry points at {0:?}")]
    DuplicatePath(String),
    /// A group toggle names a group whose members may become root. Joining
    /// and leaving it belongs on the Users page, behind its guards.
    #[error("the group {0:?} grants root, it is changed on the Users page only")]
    AdminGroup(String),
    #[error("empty name")]
    EmptyName,
    #[error("empty title")]
    EmptyTitle,
    #[error("empty description")]
    EmptyDescription,
    /// A bidi override or zero-width character would make the page read
    /// otherwise than the catalog file looks in review.
    #[error("{0:?} holds a hidden character")]
    HiddenCharacter(String),
    #[error("empty icon")]
    EmptyIcon,
    /// A section without entries would show an empty group.
    #[error("no entries")]
    NoEntries,
    #[error("{0}")]
    BadName(InvalidName),
    #[error("the requires list is empty")]
    EmptyRequirement,
    #[error("requires {0:?}, which is no entry of this catalog")]
    UnknownRequirement(String),
    #[error("requires {0:?} twice")]
    RequirementRepeats(String),
    /// `requires` names the entry's own id, so it could never be changed.
    #[error("the entry requires itself")]
    RequiresItself,
    /// Following `requires` leads back to the start. Holds the ids along the
    /// cycle, starting with the reported entry.
    #[error(
        "the requirements form a cycle: {} -> {}",
        .0.join(" -> "),
        .0.first().map_or("", String::as_str)
    )]
    RequirementCycle(Vec<String>),
    /// The `files` list is empty, so a reset would copy nothing.
    #[error("no files")]
    NoFiles,
    #[error("{0:?} is not below /etc/skel/")]
    OutsideSkel(String),
    #[error("{0:?} contains \"..\"")]
    ParentComponent(String),
    /// A user config path is absolute, climbs out of the home or is empty.
    #[error("{0}")]
    BadHomePath(InvalidHomePath),
    /// A root-owned config file is relative, climbs, or lies outside the
    /// folders the helper edits.
    #[error("{0}")]
    BadConfigPath(String),
    /// A writable root-owned entry is not in `toolbox_plan`'s write table as the
    /// kind it is (file or folder), or a line of the table has no entry.
    #[error(
        "{0:?} must be both a root-owned entry without read_only and a line of toolbox_plan::WRITABLE_CONFIG_FILES or WRITABLE_CONFIG_FOLDERS"
    )]
    NotWritable(String),
    /// A `read_only` entry is a line of the write table or a file in one
    /// of its folders.
    #[error(
        "{0:?} is read_only, so it must be neither a line of toolbox_plan::WRITABLE_CONFIG_FILES or WRITABLE_CONFIG_FOLDERS nor a file in one of those folders"
    )]
    ReadOnlyWritable(String),
    /// `read_only` belongs to a file of the user, which the helper never
    /// writes anyway.
    #[error("read_only needs a root-owned path")]
    ReadOnlyHome,
    /// An INI group, key or value or a JSON key breaks the rule of words.
    #[error("{0}")]
    BadConfigWord(InvalidConfigWord),
    /// A `user-file` without content would write an empty file.
    #[error("empty content")]
    EmptyContent,
    #[error("reload needs a user-file, user-ini or user-json check")]
    ReloadWithoutUserConfig,
    #[error("unknown reload {0:?}, known: kwin")]
    UnknownReload(String),
    #[error("restart-user-units needs a user-file, user-ini or user-json check")]
    RestartWithoutUserConfig,
    #[error("restart-user-units names {0:?} twice")]
    RestartRepeats(String),
    /// An entry for no desktop at all would never show, so `desktop` needs
    /// at least one non-blank name.
    #[error("the desktop list is empty or has an empty name")]
    EmptyDesktop,
    /// `also` adds packages to a package, so it needs a package check.
    #[error("also needs a package check")]
    AlsoWithoutPackage,
    /// An `also` list without packages adds nothing.
    #[error("the also list is empty")]
    EmptyAlso,
    /// `also` names the entry's own package, or one package or unit twice.
    #[error("also names {0:?} twice")]
    AlsoRepeats(String),
    /// A `conflicts` list without ids turns nothing off.
    #[error("the conflicts list is empty")]
    EmptyConflicts,
    #[error("conflicts names {0:?} twice")]
    ConflictRepeats(String),
    #[error("conflicts with {0:?}, which is no entry of this catalog")]
    UnknownConflict(String),
    /// `conflicts` names the entry's own id, so it could never be on.
    #[error("the entry conflicts with itself")]
    ConflictsItself,
    /// `conflicts` names an entry the entry also requires, so turning it
    /// on would turn its own requirement off.
    #[error("conflicts with {0:?}, which it also requires")]
    ConflictsRequirement(String),
    /// `icon` is no plain file name of letters, digits, `.`, `_` and `-`.
    #[error("the icon {0:?} is no plain file name")]
    BadIcon(String),
    #[error("the icon {0:?} is missing from the icon folder")]
    MissingIcon(String),
    #[error("the `also` package {0:?} is protected, so the entry could never go off")]
    ProtectedAlso(String),
    /// `url` is no `https` link with a host, or has blanks or control
    /// characters, or is longer than 2048 bytes.
    #[error("the url {0:?} is no plain https link")]
    BadUrl(String),
    #[error("{0}")]
    BadColour(InvalidRgb),
    /// A terminal palette is all 16 colours or none.
    #[error("ansi has {0} colours, it needs 16")]
    AnsiCount(usize),
    #[error("empty licence")]
    EmptyLicence,
}
