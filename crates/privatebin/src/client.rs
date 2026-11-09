//! HTTP sits behind [`Transport`], so everything but the socket is tested without a network.

#[cfg(test)]
mod tests;

use std::fmt;

use serde::Deserialize;
pub use toolbox_https::RequestError;

use crate::paste::{SealError, Secrets, seal};

/// Garuda's PrivateBin. A constant: the app never sends a report anywhere
/// else, whatever a setting, an argument or the server says.
pub const SERVER: &str = "https://bin.garudalinux.org";

/// PrivateBin sends ids of 16 hex digits and delete tokens of 64. The
/// bounds leave a margin over both, and a longer answer is no id.
const MAX_ID_CHARS: usize = 64;
const MAX_TOKEN_CHARS: usize = 128;
const MAX_MESSAGE_CHARS: usize = 200;

pub trait Transport: Send + Sync {
    /// # Errors
    ///
    /// [`RequestError`] when the server gave no answer.
    fn post(&self, url: &str, body: &str) -> Result<String, RequestError>;
}

/// `Debug` leaves `link` out: its fragment is the key, and a log line
/// with it would hand the report to whoever reads the log.
#[derive(Clone, PartialEq, Eq)]
pub struct Paste {
    /// Whoever has it can read the paste, since the key is in its fragment.
    pub link: String,
    /// Opening it deletes the paste before it expires. It holds no key, so
    /// it cannot be read with it, but whoever has it can remove the paste.
    pub delete_link: String,
}

impl fmt::Debug for Paste {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The key after `#` opens the paste and the delete link's token
        // removes it, so only the address before the key is printed.
        let address = self.link.split('#').next().unwrap_or_default();
        f.debug_struct("Paste")
            .field("address", &address)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum UploadError {
    #[error(transparent)]
    Seal(#[from] SealError),
    #[error(transparent)]
    Transport(RequestError),
    /// The server said no, for example to a paste that is too large.
    #[error("the server refused the paste: {0}")]
    Refused(String),
    #[error("the server's answer is not understood")]
    BadAnswer,
}

/// The `url` field is not read, since the link is built here.
#[derive(Deserialize)]
struct Answer {
    status: i64,
    #[serde(default)]
    id: String,
    #[serde(default)]
    deletetoken: String,
    #[serde(default)]
    message: String,
}

/// # Errors
///
/// [`UploadError`] when the text cannot be encrypted, the server gives no
/// answer, refuses the paste or answers something else.
pub fn upload(transport: &dyn Transport, text: &str) -> Result<Paste, UploadError> {
    upload_with(transport, text, &Secrets::random()?)
}

fn upload_with(
    transport: &dyn Transport,
    text: &str,
    secrets: &Secrets,
) -> Result<Paste, UploadError> {
    let body = seal(text, secrets)?
        .to_json()
        .map_err(|_unserializable| SealError::Encryption)?;

    let answer = transport
        .post(&format!("{SERVER}/"), &body)
        .map_err(|error| match error {
            RequestError::Status(status) => UploadError::Refused(format!("status {status}")),
            other @ (RequestError::NoAnswer(_)
            | RequestError::OtherHost(_)
            | RequestError::Unreadable(_)) => UploadError::Transport(other),
        })?;

    let answer: Answer =
        serde_json::from_str(&answer).map_err(|_no_json| UploadError::BadAnswer)?;

    if answer.status != 0 {
        return Err(UploadError::Refused(plain(&answer.message)));
    }

    if !is_token(&answer.id, MAX_ID_CHARS) || !is_token(&answer.deletetoken, MAX_TOKEN_CHARS) {
        return Err(UploadError::BadAnswer);
    }

    Ok(Paste {
        link: format!("{SERVER}/?{}#{}", answer.id, secrets.fragment()),
        // What PrivateBin's own page shows as "delete link" after a paste.
        delete_link: format!(
            "{SERVER}/?pasteid={}&deletetoken={}",
            answer.id, answer.deletetoken
        ),
    })
}

/// Letters and digits only, so an answer can add neither a host nor a
/// path nor another parameter to the links the app later opens.
fn is_token(text: &str, max: usize) -> bool {
    (1..=max).contains(&text.len())
        && text
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
}

/// The server's message as one short line of plain text. Control and
/// escape characters never reach the display, and neither do Unicode
/// bidi overrides, which reorder text instead of showing it.
fn plain(message: &str) -> String {
    message
        .chars()
        .filter(|character| !toolbox_plan::text::is_hidden(*character))
        .take(MAX_MESSAGE_CHARS)
        .collect()
}
