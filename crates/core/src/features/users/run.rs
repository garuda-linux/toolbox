//! A new password and a new account run at once.
//!
//! The page stays open while they do. The outcome shows on the page, with the
//! Console one click away, because switching the whole window for a few lines
//! of output loses the user.

#[cfg(test)]
mod tests;

use toolbox_plan::UserName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum RunKind {
    Password,
    NewAccount,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum PageRun {
    #[default]
    None,
    /// Sent to the helper, which has not started it. Another plan may run
    /// first and end before it.
    Sent(RunKind, UserName),
    Running(RunKind, UserName),
    Done(RunKind, UserName),
    Failed(RunKind, UserName),
}

impl RunKind {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

impl PageRun {
    /// `""`, `"running"`, `"done"` or `"failed"`, for the page's message.
    pub const fn state(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::Sent(..) | Self::Running(..) => "running",
            Self::Done(..) => "done",
            Self::Failed(..) => "failed",
        }
    }

    /// `""`, `"password"` or `"new-account"`.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::Sent(kind, _)
            | Self::Running(kind, _)
            | Self::Done(kind, _)
            | Self::Failed(kind, _) => kind.as_str(),
        }
    }

    pub fn user(&self) -> &str {
        match self {
            Self::None => "",
            Self::Sent(_, user)
            | Self::Running(_, user)
            | Self::Done(_, user)
            | Self::Failed(_, user) => user.as_str(),
        }
    }

    /// The state once a plan ended. Plans run one after the other, so the
    /// first end after the start is the end of this run.
    #[must_use = "returns the new value, the old one stays as it was"]
    pub fn finished(self, succeeded: bool) -> Self {
        match self {
            Self::Running(kind, user) if succeeded => Self::Done(kind, user),
            Self::Running(kind, user) => Self::Failed(kind, user),
            other @ (Self::None | Self::Sent(..) | Self::Done(..) | Self::Failed(..)) => other,
        }
    }

    /// The helper started the sent run.
    #[must_use = "returns the new value, the old one stays as it was"]
    pub fn started(self) -> Self {
        match self {
            Self::Sent(kind, user) => Self::Running(kind, user),
            other @ (Self::None | Self::Running(..) | Self::Done(..) | Self::Failed(..)) => other,
        }
    }

    /// The helper refused the sent run, because another plan runs.
    #[must_use = "returns the new value, the old one stays as it was"]
    pub fn refused(self) -> Self {
        match self {
            Self::Sent(..) => Self::None,
            other @ (Self::None | Self::Running(..) | Self::Done(..) | Self::Failed(..)) => other,
        }
    }
}
