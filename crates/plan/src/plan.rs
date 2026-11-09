//! An ordered list of actions and how it splits between the app and the
//! helper.

#[cfg(test)]
mod tests;

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{RootAction, UserAction};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StepId(u32);

impl StepId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

impl fmt::Display for StepId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step<A> {
    pub id: StepId,
    pub action: A,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlannedAction {
    Root(RootAction),
    User(UserAction),
}

impl PlannedAction {
    /// Whether the helper runs the step, with root rights.
    pub const fn is_root(&self) -> bool {
        matches!(self, Self::Root(_))
    }

    /// The step's command lines as the Review page shows them: the text of
    /// each part of the procedure the runner executes, in order.
    pub fn command_lines(&self) -> Vec<String> {
        let procedure = match self {
            Self::Root(action) => action.procedure(),
            Self::User(action) => action.procedure(),
        };

        procedure.iter().map(ToString::to_string).collect()
    }
}

/// A run of steps that execute in the same place: one helper run for root
/// steps, the app's runner for user steps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Segment {
    Root(Vec<Step<RootAction>>),
    User(Vec<Step<UserAction>>),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plan {
    steps: Vec<Step<PlannedAction>>,
}

impl Plan {
    #[must_use = "a plan does nothing until it is run"]
    pub fn new(actions: impl IntoIterator<Item = PlannedAction>) -> Self {
        let steps = (1_u32..)
            .zip(actions)
            .map(|(id, action)| Step {
                id: StepId(id),
                action,
            })
            .collect();

        Self { steps }
    }

    #[must_use = "a plan does nothing until it is run"]
    pub fn root(action: RootAction) -> Self {
        Self::new([PlannedAction::Root(action)])
    }

    #[must_use = "a plan does nothing until it is run"]
    pub fn user(action: UserAction) -> Self {
        Self::new([PlannedAction::User(action)])
    }

    pub fn steps(&self) -> &[Step<PlannedAction>] {
        &self.steps
    }

    pub const fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// The steps grouped into consecutive runs of the same kind, in order.
    /// Each root segment costs one helper start (and at most one password).
    /// Takes the plan: an action may hold a whole config file.
    pub fn into_segments(self) -> Vec<Segment> {
        let mut segments: Vec<Segment> = Vec::new();

        for Step { id, action } in self.steps {
            match (action, segments.last_mut()) {
                (PlannedAction::Root(action), Some(Segment::Root(steps))) => {
                    steps.push(Step { id, action });
                }
                (PlannedAction::User(action), Some(Segment::User(steps))) => {
                    steps.push(Step { id, action });
                }
                (PlannedAction::Root(action), _) => {
                    segments.push(Segment::Root(vec![Step { id, action }]));
                }
                (PlannedAction::User(action), _) => {
                    segments.push(Segment::User(vec![Step { id, action }]));
                }
            }
        }

        segments
    }
}
