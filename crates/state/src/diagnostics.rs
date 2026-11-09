//! The report is English whatever the app's language: the people who read
//! it in the forum are not the one who made it.

mod redact;
pub mod report;
mod selection;
#[cfg(test)]
mod tests;
mod upload;

pub use redact::{Kind as RedactedKind, Known, Redacted, redact};
pub use report::{Attempt, Outcome, Ran, Refusal, full_report, outcome, since_last_pacman_run};
pub use selection::Selection;
use toolbox_plan::protocol::ReadKind;
pub use upload::UploadFailure;

/// Programs colour their output for a terminal only, and the report comes
/// through a pipe. So each collector is asked for colours in its own way,
/// which the page shows and the plain report leaves out.
///
/// systemd's tools: sixteen colours, which follow the desktop's theme.
/// With `1` they print 256 colour codes that have no theme colour.
const SYSTEMD_COLORS: &[(&str, &str)] = &[("SYSTEMD_COLORS", "16")];

/// garuda-inxi decides by `[ -t 1 ]` alone and has no option for colours,
/// so its report stays plain.
const INXI: &[Program] = &[Program {
    name: "garuda-inxi",
    arguments: &[],
    environment: &[],
}];

const HEALTH: &[Program] = &[Program {
    name: "garuda-health",
    arguments: &["--forcetty"],
    environment: &[],
}];

const SYSTEMD_ANALYZE: &[Program] = &[
    Program {
        name: "systemd-analyze",
        arguments: &["blame", "--no-pager"],
        environment: SYSTEMD_COLORS,
    },
    Program {
        name: "systemd-analyze",
        arguments: &["critical-chain", "--no-pager"],
        environment: SYSTEMD_COLORS,
    },
];

const JOURNAL: &[Program] = &[Program {
    name: "journalctl",
    arguments: &["-e", "--no-pager"],
    environment: SYSTEMD_COLORS,
}];

const DMESG: &[Program] = &[Program {
    name: "dmesg",
    arguments: &["--color=always"],
    environment: &[],
}];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Collector {
    Inxi,
    Health,
    SystemdAnalyze,
    Journal,
    PacmanLog,
    Dmesg,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Access {
    /// `kernel.dmesg_restrict` is off.
    pub kernel_log: bool,
    pub journal: bool,
}

/// Fixed here, never from input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Program {
    pub name: &'static str,
    pub arguments: &'static [&'static str],
    pub environment: &'static [(&'static str, &'static str)],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// Programs run as the user, one after the other while each succeeds.
    Programs(&'static [Program]),
    Helper(ReadKind),
    /// The end of `/var/log/pacman.log`, which every user may read.
    PacmanLog,
}

impl Collector {
    pub const ALL: [Self; 6] = [
        Self::Inxi,
        Self::SystemdAnalyze,
        Self::Journal,
        Self::PacmanLog,
        Self::Dmesg,
        Self::Health,
    ];

    /// The id the UI and `--action` use, and the heading in the full report.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inxi => "inxi",
            Self::Health => "garuda-health",
            Self::SystemdAnalyze => "systemd-analyze",
            Self::Journal => "journalctl",
            Self::PacmanLog => "pacman",
            Self::Dmesg => "dmesg",
        }
    }

    /// Where the text comes from, given what the user may read. Root is
    /// asked only where the user's own rights do not reach.
    pub const fn source(self, access: Access) -> Source {
        match self {
            Self::Inxi => Source::Programs(INXI),
            Self::Health => Source::Programs(HEALTH),
            Self::SystemdAnalyze => Source::Programs(SYSTEMD_ANALYZE),
            Self::Journal if access.journal => Source::Programs(JOURNAL),
            Self::Journal => Source::Helper(ReadKind::Journal),
            Self::PacmanLog => Source::PacmanLog,
            Self::Dmesg if access.kernel_log => Source::Programs(DMESG),
            Self::Dmesg => Source::Helper(ReadKind::Dmesg),
        }
    }
}
