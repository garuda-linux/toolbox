#[cfg(test)]
mod tests;

use toolbox_privatebin::UploadError;
use toolbox_state::diagnostics::UploadFailure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UploadProblem {
    pub kind: UploadFailure,
    /// Plain text the UI shows only as such, empty when there are none.
    pub detail: String,
}

impl UploadProblem {
    pub const fn stopped() -> Self {
        Self {
            kind: UploadFailure::Stopped,
            detail: String::new(),
        }
    }
}

impl From<UploadError> for UploadProblem {
    fn from(error: UploadError) -> Self {
        let (kind, detail) = match error {
            UploadError::Transport(reason) => (UploadFailure::Network, reason.to_string()),
            UploadError::Refused(message) => (UploadFailure::Refused, message),
            UploadError::BadAnswer => (UploadFailure::BadAnswer, String::new()),
            UploadError::Seal(reason) => (UploadFailure::Seal, reason.to_string()),
        };

        Self { kind, detail }
    }
}
