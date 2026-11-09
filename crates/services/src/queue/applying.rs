//! The plan Apply made of the queue, from its sending to its end, so only
//! the end of that plan settles the queue's changes.

#[cfg(test)]
mod tests;

use toolbox_core::Owners;
use toolbox_plan::Plan;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Applying {
    #[default]
    None,
    /// Sent to the helper, which has not started it. Another plan may
    /// still run and end before it.
    Sent(Plan, Owners),
    Running(Plan, Owners),
}

impl Applying {
    /// Plans run one after the other, so the next start after the sending
    /// is this plan's start.
    pub fn started(self) -> Self {
        match self {
            Self::Sent(plan, owners) => Self::Running(plan, owners),
            other @ (Self::None | Self::Running(..)) => other,
        }
    }

    /// The helper refused the sent plan, because another one runs.
    pub fn refused(self) -> Self {
        match self {
            Self::Sent(..) => Self::None,
            other @ (Self::None | Self::Running(..)) => other,
        }
    }

    /// The plan and its owners, when the plan that ended is the queue's.
    /// `None` for the end of another plan.
    pub fn finished(&mut self) -> Option<(Plan, Owners)> {
        match std::mem::take(self) {
            Self::Running(plan, owners) => Some((plan, owners)),
            other @ (Self::None | Self::Sent(..)) => {
                *self = other;
                None
            }
        }
    }
}
