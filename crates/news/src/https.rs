//! The real [`Transport`]: one HTTPS GET through `toolbox-https`.

use std::io::Read as _;
use std::time::Duration;

use toolbox_https::http::StatusCode;
use toolbox_https::{RequestError, Rules};

use crate::Transport;

/// The feed is a few dozen KiB. A dead server does not hold Home's read
/// for longer.
const TIMEOUT: Duration = Duration::from_secs(30);

/// Far more than any feed of a few posts.
const MAX_BYTES: u64 = 1 << 20;

/// Plain and generic: the forum learns the program, nothing of the user.
const USER_AGENT: &str = concat!("garuda-toolbox/", env!("CARGO_PKG_VERSION"));

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
                max_body_bytes: MAX_BYTES,
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
    fn get(&self, url: &str) -> Result<Vec<u8>, RequestError> {
        let response = self.https.get(url, &[])?;
        if response.status() != StatusCode::OK {
            return Err(RequestError::Status(response.status().as_u16()));
        }

        // A body past the cap reads as no answer, as a broken connection.
        let mut body = Vec::new();
        self.https
            .reader(response)
            .read_to_end(&mut body)
            .map_err(|error| RequestError::NoAnswer(error.to_string()))?;

        Ok(body)
    }
}
