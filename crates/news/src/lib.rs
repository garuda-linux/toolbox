//! News of Garuda Linux and Arch Linux, read from their RSS feeds only:
//! fixed addresses, nothing built from input. A [`Cache`] per feed
//! downloads it at most every [`REFRESH_EVERY`] and keeps its posts in the
//! cache folder, [`Feeds`] answers with the posts of both, newest first,
//! so Home shows the last posts offline too.
//!
//! HTTP sits behind [`Transport`] and time behind [`Clock`], so the rules
//! are tested without a network. [`Https`] is the real transport.

mod cache;
mod feed;
mod https;
mod merged;
mod source;

use std::time::Duration;

pub use cache::Cache;
pub use feed::Post;
pub use https::Https;
pub use merged::Feeds;
use serde::{Deserialize, Serialize};
pub use source::Source;
pub use toolbox_https::{Clock, RequestError, SystemClock};

/// How long an attempt, failed or not, holds the next one back.
pub const REFRESH_EVERY: Duration = Duration::from_hours(6);

/// The posts of the last download that worked.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct News {
    pub posts: Vec<Post>,
    /// Seconds since the epoch.
    pub downloaded: i64,
}

pub trait NewsSource: Send + Sync {
    /// The local copy, downloaded first when the last attempt is older
    /// than [`REFRESH_EVERY`]. Blocks while it downloads, so call it off
    /// the frame.
    ///
    /// # Errors
    ///
    /// [`NewsError`] when there is no copy to answer with.
    fn news(&self) -> Result<News, NewsError>;
}

pub trait Transport: Send + Sync {
    /// # Errors
    ///
    /// [`RequestError`] when the server gave no usable answer. Any status
    /// but 200 is [`RequestError::Status`], a redirect too.
    fn get(&self, url: &str) -> Result<Vec<u8>, RequestError>;
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum NewsError {
    /// No copy yet and the download failed now.
    #[error("the forum's news could not be downloaded: {0}")]
    Download(String),
    /// No copy yet, and the last attempt was too recent to try again.
    #[error("the forum's news is not downloaded again before {next}")]
    Waiting {
        /// Seconds since the epoch when the next attempt is allowed.
        next: i64,
    },
    #[error("the news copy could not be stored: {0}")]
    Storage(String),
}
