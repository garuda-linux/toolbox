//! The real [`Transport`]: one conditional HTTPS GET through
//! `toolbox-https`.

#[cfg(test)]
mod tests;

use std::time::Duration;

use toolbox_https::http::{HeaderMap, StatusCode, header};
use toolbox_https::{RequestError, Rules};

use crate::transport::{Answer, Transport, Validators};

/// About 14 MB over a slow line. A dead server does not hold the read
/// for longer.
const TIMEOUT: Duration = Duration::from_mins(5);

/// The dump is about 14 MB gzipped. Far more is no dump of the AUR.
const MAX_GZIPPED_BYTES: u64 = 64 << 20;

/// Plain and generic: the AUR learns the program, nothing of the user.
const USER_AGENT: &str = concat!("garuda-toolbox/", env!("CARGO_PKG_VERSION"));

/// The host of [`crate::DUMP_URL`].
pub const HOST: &str = "aur.archlinux.org";

/// A validator longer than this is no `ETag` or date and is not kept.
const MAX_VALIDATOR_BYTES: usize = 256;

/// Blocks while a request runs, so call it off the GUI thread.
pub struct Https {
    https: toolbox_https::Https,
}

impl Https {
    #[must_use = "the transport does nothing until a request is sent"]
    pub fn new() -> Self {
        Self::with_timeout(TIMEOUT)
    }

    /// A transport that gives up on a request after `timeout`, for a
    /// caller someone waits on.
    #[must_use = "the transport does nothing until a request is sent"]
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            https: toolbox_https::Https::new(&Rules {
                host: HOST,
                timeout,
                max_body_bytes: MAX_GZIPPED_BYTES,
                user_agent: Some(USER_AGENT),
            }),
        }
    }
}

impl Default for Https {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for Https {
    fn get(&self, url: &str, validators: &Validators) -> Result<Answer, RequestError> {
        let mut headers = Vec::new();
        if let Some(etag) = &validators.etag {
            headers.push((header::IF_NONE_MATCH, etag.as_str()));
        }

        if let Some(date) = &validators.last_modified {
            headers.push((header::IF_MODIFIED_SINCE, date.as_str()));
        }

        let response = match self.https.get(url, &headers) {
            Ok(response) => response,
            Err(RequestError::Status(304)) => return Ok(Answer::NotModified),
            Err(error) => return Err(error),
        };

        match response.status() {
            StatusCode::OK => {
                let answered = Validators {
                    etag: validator(response.headers(), header::ETAG),
                    last_modified: validator(response.headers(), header::LAST_MODIFIED),
                };

                Ok(Answer::Fresh {
                    body: Box::new(self.https.reader(response)),
                    validators: answered,
                })
            }
            other => Err(RequestError::Status(other.as_u16())),
        }
    }
}

/// A header's value when it is short visible ASCII, as validators are.
fn validator(headers: &HeaderMap, name: header::HeaderName) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .filter(|value| {
            value.len() <= MAX_VALIDATOR_BYTES
                && value
                    .chars()
                    .all(|character| character.is_ascii_graphic() || character == ' ')
        })
        .map(str::to_owned)
}
