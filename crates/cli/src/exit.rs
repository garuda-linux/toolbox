//! The exit codes scripts can rely on.
//!
//! They are part of the interface: a code once given keeps its meaning, a
//! new case gets a new code. 7 and 10 are retired: a change that would do
//! nothing ends with 0 and a signal with 128 and its number, as a shell
//! reports it.

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    Success,
    Failure,
    /// The arguments were wrong. clap uses this code too.
    Usage,
    Declined,
    /// A change needs `--yes`, since there is no terminal to ask on.
    NeedsYes,
    Unavailable,
    Refused,
    /// The page would refuse the same click, such as removing root.
    PageRule,
    /// `status` or `doctor` found something to do: updates, files to
    /// merge, a health warning or error, a pending restart, or a part the
    /// toolbox needs that is missing or broken.
    Attention,
    /// SIGHUP stopped it, such as a closed terminal.
    HungUp,
    Interrupted,
    Terminated,
}

impl Exit {
    pub const ALL: [Self; 12] = [
        Self::Success,
        Self::Failure,
        Self::Usage,
        Self::Declined,
        Self::NeedsYes,
        Self::Unavailable,
        Self::Refused,
        Self::PageRule,
        Self::Attention,
        Self::HungUp,
        Self::Interrupted,
        Self::Terminated,
    ];

    pub const fn code(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::Failure => 1,
            Self::Usage => 2,
            Self::Declined => 3,
            Self::NeedsYes => 4,
            Self::Unavailable => 5,
            Self::Refused => 6,
            Self::PageRule => 8,
            Self::Attention => 9,
            Self::HungUp => 129,
            Self::Interrupted => 130,
            Self::Terminated => 143,
        }
    }

    /// The code's stable name for scripts, `reason` in a read's JSON error.
    pub const fn reason(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
            Self::Usage => "usage",
            Self::Declined => "declined",
            Self::NeedsYes => "needs-yes",
            Self::Unavailable => "unavailable",
            Self::Refused => "denied",
            Self::PageRule => "page-rule",
            Self::Attention => "attention",
            Self::HungUp => "hung-up",
            Self::Interrupted => "interrupted",
            Self::Terminated => "terminated",
        }
    }

    pub const fn meaning(self) -> &'static str {
        match self {
            Self::Success => "done",
            Self::Failure => "a change or a read failed",
            Self::Usage => "the arguments were wrong",
            Self::Declined => "you answered no",
            Self::NeedsYes => "no terminal to ask on, pass --yes",
            Self::Unavailable => "not available on this system",
            Self::Refused => "the password dialog or the helper turned it down",
            Self::PageRule => "refused by the page's rules",
            Self::Attention => "status or doctor found something that needs attention",
            Self::HungUp => "SIGHUP stopped it, as Ctrl+C does",
            Self::Interrupted => "Ctrl+C stopped it",
            Self::Terminated => "SIGTERM stopped it, as Ctrl+C does",
        }
    }

    /// The man page's end: every code, its reason and its meaning. The
    /// help leaves it out.
    pub fn table() -> String {
        let wide = Self::ALL
            .iter()
            .map(|exit| exit.reason().len())
            .max()
            .unwrap_or(0);

        let lines: Vec<String> = Self::ALL
            .iter()
            .map(|exit| {
                format!(
                    "  {:<3}  {:<wide$}  {}",
                    exit.code(),
                    exit.reason(),
                    exit.meaning()
                )
            })
            .collect();

        format!("Exit codes:\n{}", lines.join("\n"))
    }
}

impl From<Exit> for std::process::ExitCode {
    fn from(exit: Exit) -> Self {
        Self::from(exit.code())
    }
}
