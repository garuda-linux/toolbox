//! The checks of a request that run before its first step.

use std::collections::HashSet;

use super::{MAX_STEPS, PROTOCOL_VERSION, Request, RequestError};
use crate::{RootAction, Step, protection};

impl Request {
    /// # Errors
    ///
    /// [`RequestError`] for a version mismatch, no steps, too many steps,
    /// a step id used twice or the removal of a protected package.
    pub fn validate(&self) -> Result<(), RequestError> {
        if self.version != PROTOCOL_VERSION {
            return Err(RequestError::VersionMismatch {
                expected: PROTOCOL_VERSION,
                got: self.version,
            });
        }

        if self.steps.is_empty() {
            return Err(RequestError::NoSteps);
        }

        if self.steps.len() > MAX_STEPS {
            return Err(RequestError::TooManySteps { max: MAX_STEPS });
        }

        let mut seen = HashSet::new();
        if let Some(step) = self.steps.iter().find(|step| !seen.insert(step.id)) {
            return Err(RequestError::DuplicateStep(step.id));
        }

        self.check_gates()?;

        self.steps.iter().try_for_each(refuse_protected)
    }

    fn check_gates(&self) -> Result<(), RequestError> {
        let first = self.steps.first().map(|step| step.id);
        let mut gated = HashSet::new();

        for gate in &self.gates {
            if !self.steps.iter().any(|step| step.id == *gate) {
                return Err(RequestError::UnknownGate(*gate));
            }

            if Some(*gate) == first {
                return Err(RequestError::GateOnFirstStep(*gate));
            }

            if !gated.insert(*gate) {
                return Err(RequestError::DuplicateGate(*gate));
            }
        }

        Ok(())
    }
}

fn refuse_protected(step: &Step<RootAction>) -> Result<(), RequestError> {
    let RootAction::RemovePackages { packages } = &step.action else {
        return Ok(());
    };

    packages
        .iter()
        .find(|package| protection(package).is_some())
        .map_or(Ok(()), |package| {
            Err(RequestError::Protected {
                step: step.id,
                package: package.clone(),
            })
        })
}
