//! The outdated AUR packages a test is given, in place of `paru -Qua`.

use crate::{AurUpdateSource, PendingUpdate, ProbeError};

/// The answer of `paru -Qua`: the pending updates, or the failure it gives.
#[derive(Debug)]
pub struct FakeAurUpdates(Result<Vec<PendingUpdate>, &'static str>);

impl FakeAurUpdates {
    pub const fn new(answer: Result<Vec<PendingUpdate>, &'static str>) -> Self {
        Self(answer)
    }
}

impl AurUpdateSource for FakeAurUpdates {
    fn pending_aur(&self) -> Result<Vec<PendingUpdate>, ProbeError> {
        match &self.0 {
            Ok(updates) => Ok(updates.clone()),
            Err(reason) => Err(ProbeError::fake(*reason)),
        }
    }
}
