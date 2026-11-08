//! The plan Apply made of the queue, from its sending to its end, so only
//! the end of that plan settles the queue's one-off work.

#[cfg(test)]
mod tests;

use toolbox_plan::{Plan, PlannedAction, StepId};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Applying {
    #[default]
    None,
    /// Sent to the helper, which has not started it. Another plan may
    /// still run and end before it.
    Sent(Plan),
    Running(Plan),
}

impl Applying {
    /// Plans run one after the other, so the next start after the sending
    /// is this plan's start.
    pub fn started(self) -> Self {
        match self {
            Self::Sent(plan) => Self::Running(plan),
            other @ (Self::None | Self::Running(_)) => other,
        }
    }

    /// The helper refused the sent plan, because another one runs.
    pub fn refused(self) -> Self {
        match self {
            Self::Sent(_) => Self::None,
            other @ (Self::None | Self::Running(_)) => other,
        }
    }

    /// The actions of the steps `succeeded` names, when the plan that
    /// ended is the queue's. `None` for the end of another plan.
    pub fn finished(&mut self, succeeded: impl Fn(StepId) -> bool) -> Option<Vec<PlannedAction>> {
        if !matches!(self, Self::Running(_)) {
            return None;
        }

        let Self::Running(plan) = std::mem::take(self) else {
            return None;
        };

        Some(
            plan.steps()
                .iter()
                .filter(|step| succeeded(step.id))
                .map(|step| step.action.clone())
                .collect(),
        )
    }
}
