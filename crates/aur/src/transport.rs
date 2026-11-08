use std::fmt;
use std::io::Read;

use serde::{Deserialize, Serialize};
use toolbox_https::RequestError;

/// What the last download's answer said of its copy, sent back so the
/// server can answer "not modified" instead of the whole dump.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validators {
    /// `ETag`, sent back as `If-None-Match`.
    pub etag: Option<String>,
    /// `Last-Modified`, sent back as `If-Modified-Since`.
    pub last_modified: Option<String>,
}

/// The server's answer to one GET.
pub enum Answer {
    /// 304: the copy is current.
    NotModified,
    /// 200 with the gzipped dump.
    Fresh {
        body: Box<dyn Read + Send>,
        validators: Validators,
    },
}

pub trait Transport: Send + Sync {
    /// # Errors
    ///
    /// [`RequestError`] when the server gave no usable answer.
    /// Any status but 200 and 304 is [`RequestError::Status`], a redirect
    /// too.
    fn get(&self, url: &str, validators: &Validators) -> Result<Answer, RequestError>;
}

impl fmt::Debug for Answer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotModified => f.write_str("NotModified"),
            Self::Fresh {
                body: _,
                validators,
            } => f
                .debug_struct("Fresh")
                .field("validators", validators)
                .finish_non_exhaustive(),
        }
    }
}
