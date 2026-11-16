//! The real [`Transport`]: one HTTPS POST through `toolbox-https`.

#[cfg(test)]
mod tests;

use std::time::Duration;

use toolbox_https::http::HeaderName;
use toolbox_https::http::header::CONTENT_TYPE;
use toolbox_https::{RequestError, Rules, USER_AGENT};

use crate::client::Transport;

/// Fits a large paste on a slow line, and keeps a dead server from holding the page.
const TIMEOUT: Duration = Duration::from_mins(2);

/// Must match the host of [`crate::client::SERVER`], which a test checks.
pub const HOST: &str = "bin.garudalinux.org";

/// A paste answer is one short JSON object.
const MAX_ANSWER_BYTES: u64 = 64 << 10;

/// PrivateBin answers JSON only to this header.
const REQUESTED_WITH: HeaderName = HeaderName::from_static("x-requested-with");

/// Blocks while a request runs, so call it off the GUI thread.
pub struct Https {
    https: toolbox_https::Https,
}

impl Https {
    #[must_use = "the transport does nothing until a request is sent"]
    pub fn new() -> Self {
        Self {
            https: toolbox_https::Https::new(&Rules {
                host: HOST,
                timeout: TIMEOUT,
                max_body_bytes: MAX_ANSWER_BYTES,
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
    fn post(&self, url: &str, body: &str) -> Result<String, RequestError> {
        let response = self.https.post(
            url,
            &[
                (CONTENT_TYPE, "application/json"),
                (REQUESTED_WITH, "JSONHttpRequest"),
            ],
            body,
        )?;

        self.https.text(response)
    }
}
