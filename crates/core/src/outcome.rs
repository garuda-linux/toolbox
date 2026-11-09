//! What the last apply did for each queued change. A domain's first step
//! can already make the probe see the wish, as a `locale.gen` edit does
//! before `locale-gen` runs. Only the outcome tells such a change from one
//! whose plan finished.

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use toolbox_plan::StepId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    #[default]
    Untried,
    Succeeded,
    /// `step` is the first step of the change's domain that did not end
    /// well. A probe that sees the wish does not drop such a change.
    Failed {
        step: StepId,
    },
}

impl Outcome {
    pub const fn is_untried(&self) -> bool {
        matches!(self, Self::Untried)
    }

    pub const fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { step: _ })
    }
}

/// Which queued changes each step of a plan carries out. A domain plans
/// its changes together (one `locale-gen` for every locale), so each of
/// its steps belongs to all of them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Owners {
    domains: Vec<Owner>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Owner {
    domain: &'static str,
    keys: Vec<String>,
    steps: Vec<StepId>,
}

impl Owners {
    /// From each step's domain and the keys each domain had queued.
    pub(crate) fn new(
        steps: impl IntoIterator<Item = (StepId, &'static str)>,
        keys: impl Fn(&'static str) -> Vec<String>,
    ) -> Self {
        let mut domains: Vec<Owner> = Vec::new();

        for (step, domain) in steps {
            match domains.iter_mut().find(|owner| owner.domain == domain) {
                Some(owner) => owner.steps.push(step),
                None => domains.push(Owner {
                    domain,
                    keys: keys(domain),
                    steps: vec![step],
                }),
            }
        }

        Self { domains }
    }

    /// Each domain, its keys and their outcome when `ended_well` names
    /// the steps that succeeded or found nothing to do.
    pub(crate) fn outcomes(
        &self,
        ended_well: impl Fn(StepId) -> bool,
    ) -> impl Iterator<Item = (&'static str, &[String], Outcome)> {
        self.domains.iter().map(move |owner| {
            let outcome = owner
                .steps
                .iter()
                .copied()
                .find(|step| !ended_well(*step))
                .map_or(Outcome::Succeeded, |step| Outcome::Failed { step });

            (owner.domain, owner.keys.as_slice(), outcome)
        })
    }
}
