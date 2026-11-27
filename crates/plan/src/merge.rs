//! The copies pacman leaves beside its backup files.

#[cfg(test)]
mod tests;

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::config_file::{ConfigFilePath, ConfigText, Fingerprint, InvalidConfigFilePath};
use crate::procedure::{Command, Program};
use crate::shown::Shown;

/// pacman's local database, relative to `/`.
pub const PACMAN_LOCAL: &str = "var/lib/pacman/local";

pub const PACKAGE_FILES: &str = "files";

pub const SNAPSHOT_DESCRIPTION: &str = "Garuda Toolbox: before merging configuration files";

pub const SNAPPER_ROOT_CONFIG: &str = "etc/snapper/configs/root";

const BACKUP_SECTION: &str = "%BACKUP%";

const SUDOERS: &str = "/etc/sudoers";

const SUDOERS_D: &str = "/etc/sudoers.d/";

const MERGE_WHY: &str =
    "only when pacman lists the file as a backup file and neither file changed meanwhile";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CopyKind {
    /// `.pacnew`
    New,
    /// `.pacsave`
    Saved,
    /// `.pacsave.1` and up: pacdiff leaves them alone, only removing one
    /// makes sense.
    OlderSave,
    /// `.pacorig`
    Orig,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidMergeCopy {
    #[error(transparent)]
    Path(#[from] InvalidConfigFilePath),
    #[error("`{}` is no copy pacman leaves beside a configuration file", Shown::new(.0))]
    NoCopy(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Resolution {
    /// Removes the copy.
    KeepCurrent,
    /// Puts the copy in the file's place.
    TakeCopy,
    /// Writes the text in the file's place and removes the copy.
    Merged { text: ConfigText },
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct MergeCopy {
    path: ConfigFilePath,
    original: ConfigFilePath,
    kind: CopyKind,
}

/// What to do with a copy, and the marks of both files as the user saw
/// them. `current` is `None` when the file in use was gone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MergeChoice {
    pub resolution: Resolution,
    pub copy: Fingerprint,
    pub current: Option<Fingerprint>,
}

/// Decoding refuses a choice the copy does not allow, see
/// [`MergeChoice::fits`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawMergeResolve")]
pub struct MergeResolve {
    pub copy: MergeCopy,
    pub choice: MergeChoice,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("`{}` cannot be resolved this way", Shown::new(.0.path.as_str()))]
pub struct UnfitChoice(MergeCopy);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMergeResolve {
    copy: MergeCopy,
    choice: MergeChoice,
}

impl CopyKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Saved => "saved",
            Self::OlderSave => "older-save",
            Self::Orig => "orig",
        }
    }
}

impl MergeCopy {
    /// # Errors
    ///
    /// [`InvalidMergeCopy`] for a path that is no [`ConfigFilePath`], or
    /// whose name does not end like one of pacman's copies.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidMergeCopy> {
        let path = ConfigFilePath::new(value)?;
        let Some((kind, original)) = split_copy(path.as_str()) else {
            return Err(InvalidMergeCopy::NoCopy(path.to_string()));
        };

        let original = ConfigFilePath::new(original)?;

        Ok(Self {
            path,
            original,
            kind,
        })
    }

    pub const fn path(&self) -> &ConfigFilePath {
        &self.path
    }

    /// The file in use the copy belongs to.
    pub const fn original(&self) -> &ConfigFilePath {
        &self.original
    }

    pub const fn kind(&self) -> CopyKind {
        self.kind
    }

    pub fn as_str(&self) -> &str {
        self.path.as_str()
    }
}

impl fmt::Debug for MergeCopy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.path.as_str(), f)
    }
}

impl fmt::Display for MergeCopy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.path.as_str())
    }
}

impl TryFrom<String> for MergeCopy {
    type Error = InvalidMergeCopy;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl AsRef<str> for MergeCopy {
    fn as_ref(&self) -> &str {
        self.path.as_str()
    }
}

impl From<MergeCopy> for String {
    fn from(copy: MergeCopy) -> Self {
        copy.path.as_str().to_owned()
    }
}

impl MergeChoice {
    /// An older save can only go, and a text can only be merged with a
    /// file that is there.
    pub const fn fits(&self, copy: &MergeCopy) -> bool {
        let older = matches!(copy.kind, CopyKind::OlderSave);

        match &self.resolution {
            Resolution::KeepCurrent => true,
            Resolution::TakeCopy => !older,
            Resolution::Merged { text: _ } => !older && self.current.is_some(),
        }
    }
}

impl TryFrom<RawMergeResolve> for MergeResolve {
    type Error = UnfitChoice;

    fn try_from(raw: RawMergeResolve) -> Result<Self, Self::Error> {
        if !raw.choice.fits(&raw.copy) {
            return Err(UnfitChoice(raw.copy));
        }

        Ok(Self {
            copy: raw.copy,
            choice: raw.choice,
        })
    }
}

impl MergeResolve {
    /// The step as the Console shows it, without its note.
    pub fn command(&self) -> String {
        let copy = &self.copy.path;
        let original = &self.copy.original;
        let backup = original.backup();

        let command = match (&self.choice.resolution, self.choice.current) {
            (Resolution::KeepCurrent, _) => format!("rm {copy}"),
            (Resolution::TakeCopy, None) => format!("mv {copy} {original}"),
            (Resolution::TakeCopy, Some(_)) => {
                format!("replace {original} with {copy}, the old one kept as /{backup}")
            }
            (Resolution::Merged { text }, _) => format!(
                "write {original} ({} bytes, merged with {copy}), the old one kept as /{backup}, then rm {copy}",
                text.as_str().len()
            ),
        };

        match self.syntax_check() {
            Some(check) => format!(
                "{command}, if {} -c accepts the new text",
                check.program().name()
            ),
            None => command,
        }
    }

    /// sudo refuses every rule while one sudoers file does not parse, so a
    /// broken one would lock out the repair.
    pub fn syntax_check(&self) -> Option<Command> {
        let puts_text = !matches!(self.choice.resolution, Resolution::KeepCurrent);
        let original = self.copy.original.as_str();
        let sudoers = original == SUDOERS
            || original
                .strip_prefix(SUDOERS_D)
                .is_some_and(|name| !name.contains('/'));

        (puts_text && sudoers).then(|| Command::new(Program::Visudo).args(["-c", "-f", "-"]))
    }

    pub const fn why() -> &'static str {
        MERGE_WHY
    }
}

impl fmt::Display for MergeResolve {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}  # {MERGE_WHY}", self.command())
    }
}

/// The paths of a `files` list's `%BACKUP%` part, relative to `/`.
pub fn backup_files(files: &str) -> impl Iterator<Item = &str> {
    files
        .lines()
        .skip_while(|line| *line != BACKUP_SECTION)
        .skip(1)
        .take_while(|line| !line.is_empty())
        .filter_map(|line| line.split('\t').next())
}

pub fn lists_backup(files: &str, original: &ConfigFilePath) -> bool {
    backup_files(files).any(|path| path == original.relative())
}

/// `--no-dbus`, as the helper runs as root without a session. The number
/// cleanup removes old snapshots of this kind, as it does for snap-pac's.
pub fn snapshot() -> Command {
    Command::new(Program::Snapper).args([
        "--no-dbus",
        "-c",
        "root",
        "create",
        "--cleanup-algorithm",
        "number",
        "--description",
        SNAPSHOT_DESCRIPTION,
    ])
}

fn split_copy(path: &str) -> Option<(CopyKind, &str)> {
    let suffixes = [
        (".pacnew", CopyKind::New),
        (".pacsave", CopyKind::Saved),
        (".pacorig", CopyKind::Orig),
    ];

    if let Some((original, kind)) = suffixes
        .iter()
        .find_map(|(suffix, kind)| path.strip_suffix(suffix).map(|original| (original, *kind)))
    {
        return Some((kind, original));
    }

    let (rest, number) = path.rsplit_once('.')?;
    let original = rest.strip_suffix(".pacsave")?;

    (!number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit()))
        .then_some((CopyKind::OlderSave, original))
}
