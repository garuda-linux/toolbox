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

/// A form kept while the run it sent goes, so a failed run can give it
/// back. The form holds no password: the run took the only copy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Retry<F> {
    form: F,
    kind: RunKind,
    user: UserName,
    /// The run of this form was seen sent or running. An outcome before
    /// that belongs to an earlier try.
    started: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RetryStep<F> {
    Wait(Retry<F>),
    Reopen(F),
    Forget,
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

    /// The state once a plan ended. The window hands on only the start
    /// reported under this run's plan id, and plans run one after the
    /// other, so the first end after that start is the end of this run.
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

impl<F> Retry<F> {
    pub const fn new(form: F, kind: RunKind, user: UserName) -> Self {
        Self {
            form,
            kind,
            user,
            started: false,
        }
    }

    /// What becomes of the form once the page's run is `run`: back after a
    /// failure or a refusal, gone after success.
    #[must_use = "the form is inside the step"]
    pub fn step(self, run: &PageRun) -> RetryStep<F> {
        match run {
            PageRun::Sent(kind, user) | PageRun::Running(kind, user) if self.is(*kind, user) => {
                RetryStep::Wait(Self {
                    started: true,
                    ..self
                })
            }
            PageRun::Failed(kind, user) if self.started && self.is(*kind, user) => {
                RetryStep::Reopen(self.form)
            }
            PageRun::None if self.started => RetryStep::Reopen(self.form),
            PageRun::Done(kind, user) if self.started && self.is(*kind, user) => RetryStep::Forget,
            PageRun::None
            | PageRun::Sent(..)
            | PageRun::Running(..)
            | PageRun::Done(..)
            | PageRun::Failed(..) => RetryStep::Wait(self),
        }
    }

    fn is(&self, kind: RunKind, user: &UserName) -> bool {
        self.kind == kind && self.user == *user
    }
}
