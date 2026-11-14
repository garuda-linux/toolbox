//! What `dkms status` answered, for the Kernels page and the command line.

#[cfg(test)]
mod tests;

use toolbox_core::features::kernels::DkmsEntry;
use toolbox_system::ProbeError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Dkms {
    NotInstalled,
    Modules(Vec<DkmsEntry>),
}

impl Dkms {
    pub fn entries(&self) -> &[DkmsEntry] {
        match self {
            Self::NotInstalled => &[],
            Self::Modules(entries) => entries,
        }
    }
}

/// dkms ran but gave no usable answer. The page shows the reason as dkms
/// gave it, without the probe's own prefix.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct DkmsFailure(pub String);

/// # Errors
///
/// [`DkmsFailure`], the reason dkms failed with.
pub fn dkms_answer(answer: Result<Vec<DkmsEntry>, ProbeError>) -> Result<Dkms, DkmsFailure> {
    match answer {
        Ok(entries) => Ok(Dkms::Modules(entries)),
        Err(ProbeError::Unavailable) => Ok(Dkms::NotInstalled),
        Err(error) => {
            let reason = error.reason().unwrap_or_default();
            tracing::warn!("dkms status: {reason}");
            Err(DkmsFailure(reason))
        }
    }
}
