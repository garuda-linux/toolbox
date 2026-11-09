#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use super::{DecodeError, MAX_READ_OUTPUT_BYTES, decode_within};
use crate::procedure::Program;

/// A closed list: each kind reads one fixed file or runs one fixed program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadKind {
    ShadowLocks,
    /// Answers a [`ProgramAnswer`]. Needed where `kernel.dmesg_restrict` keeps it from users.
    Dmesg,
    /// Answers a [`ProgramAnswer`]. Needed by users outside the groups that may read it.
    Journal,
    /// Titles and ids only, never the text of `grub.cfg`, which may hold a password hash.
    GrubMenu,
    SdBootMenu,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ProgramAnswer {
    /// The program ran to its end, whatever its exit code.
    Ran {
        /// What it printed, lossy UTF-8, at most [`MAX_READ_OUTPUT_BYTES`].
        output: String,
        errors: String,
        /// `None` when a signal ended it.
        code: Option<i32>,
        /// The start of the output was left out.
        truncated: bool,
    },
    Missing,
}

impl ProgramAnswer {
    /// The longest answer line. Control characters grow sixfold as JSON
    /// escapes and a byte that is no text threefold as its replacement.
    pub const MAX_LINE_BYTES: usize = 8 * MAX_READ_OUTPUT_BYTES;

    /// Decodes the helper's answer line.
    ///
    /// # Errors
    ///
    /// [`DecodeError`] for an overlong line or anything but an answer.
    pub fn parse(line: &str) -> Result<Self, DecodeError> {
        decode_within(line, Self::MAX_LINE_BYTES)
    }
}

impl ReadKind {
    pub const ALL: [Self; 5] = [
        Self::ShadowLocks,
        Self::Dmesg,
        Self::Journal,
        Self::GrubMenu,
        Self::SdBootMenu,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ShadowLocks => "shadow-locks",
            Self::Dmesg => "dmesg",
            Self::Journal => "journal",
            Self::GrubMenu => "grub-menu",
            Self::SdBootMenu => "sdboot-menu",
        }
    }

    /// Fixed here, so the app can name a kind and nothing else.
    pub const fn program(self) -> Option<(Program, &'static [&'static str])> {
        match self {
            Self::ShadowLocks | Self::GrubMenu | Self::SdBootMenu => None,
            // Colours are for a terminal, and the helper reads through a
            // pipe. The app shows them and takes them out of the plain report.
            Self::Dmesg => Some((Program::Dmesg, &["--color=always"])),
            // `-e` alone ends at the newest entries and implies the last
            // 1000 lines.
            Self::Journal => Some((Program::Journalctl, &["-e", "--no-pager"])),
        }
    }

    /// Fixed here like the arguments.
    pub const fn environment(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::ShadowLocks | Self::Dmesg | Self::GrubMenu | Self::SdBootMenu => &[],
            // journalctl has no option for colours. Sixteen colours, so the
            // app can show them in the desktop's theme.
            Self::Journal => &[("SYSTEMD_COLORS", "16")],
        }
    }

    /// polkit tells the helper's commands apart by this argument alone, and an active session
    /// runs `read` without a password. Whole logs need a command of their own, which that rule cannot match.
    pub const fn command(self) -> &'static str {
        match self {
            Self::ShadowLocks | Self::GrubMenu | Self::SdBootMenu => "read",
            Self::Dmesg | Self::Journal => "logs",
        }
    }

    /// A kind under the other command is none, so `read journal` never runs.
    pub fn from_args(command: &str, kind: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|candidate| candidate.command() == command && candidate.as_str() == kind)
    }
}
