//! The real [`Transport`]: one HTTPS POST through `toolbox-https`.

use std::time::Duration;

use toolbox_https::http::HeaderName;
use toolbox_https::http::header::CONTENT_TYPE;
use toolbox_https::{RequestError, Rules};

use crate::client::Transport;

/// A report of some megabytes over a slow line still fits. A dead server
/// does not block the page for longer.
const TIMEOUT: Duration = Duration::from_mins(2);

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
                timeout: TIMEOUT,
                max_body_bytes: MAX_ANSWER_BYTES,
                // ureq's own: the paste server was never told the program.
                user_agent: None,
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
