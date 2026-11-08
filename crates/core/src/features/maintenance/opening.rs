//! A tool that was missing is opened once the install of its package ends
//! well, so "Install and Open…" is one click.

#[cfg(test)]
mod tests;

use super::Tool;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Next {
    Tool(Tool),
    /// The ranking of the Chaotic-AUR mirrors, which needed rate-mirrors.
    RankMirrors,
}

/// The install a tool waits for. Another plan's end must not open it, so
/// the wait only moves on with the start of the plan this page sent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Opening {
    #[default]
    None,
    /// The install was sent and has not started yet.
    Sent(Next),
    Installing(Next),
}

impl Opening {
    #[must_use = "returns the new value, the old one stays as it was"]
    pub const fn started(self) -> Self {
        match self {
            Self::Sent(next) => Self::Installing(next),
            other @ (Self::None | Self::Installing(_)) => other,
        }
    }

    #[must_use = "returns the new value, the old one stays as it was"]
    pub const fn refused(self) -> Self {
        match self {
            Self::Sent(_) => Self::None,
            other @ (Self::None | Self::Installing(_)) => other,
        }
    }

    pub const fn finished(self, succeeded: bool) -> (Option<Next>, Self) {
        match self {
            Self::Installing(next) if succeeded => (Some(next), Self::None),
            Self::Installing(_) => (None, Self::None),
            other @ (Self::None | Self::Sent(_)) => (None, other),
        }
    }
}
