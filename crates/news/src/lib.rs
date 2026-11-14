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

use std::sync::Arc;
use std::time::Duration;

pub use cache::Cache;
pub use feed::{FeedError, Post};
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

/// Equal when of one kind and read the same, as an I/O or JSON error has
/// no equality of its own.
#[derive(Clone, Debug, thiserror::Error)]
pub enum NewsError {
    /// No copy yet and the download failed now.
    #[error("the forum's news could not be downloaded: {0}")]
    Request(#[source] RequestError),
    /// No copy yet and the feed that came now is no valid XML.
    #[error("the forum's news could not be downloaded: {0}")]
    Feed(#[source] FeedError),
    /// No feed answered and none named why.
    #[error("the forum's news could not be downloaded: no feed")]
    NoFeed,
    /// No copy yet, and the last attempt was too recent to try again.
    #[error("the forum's news is not downloaded again before {next}")]
    Waiting {
        /// Seconds since the epoch when the next attempt is allowed.
        next: i64,
    },
    #[error("the news copy could not be stored: {error}")]
    Storage {
        step: StorageStep,
        #[source]
        error: Arc<std::io::Error>,
    },
    #[error("the news copy could not be stored: {0}")]
    Encode(#[source] Arc<serde_json::Error>),
}

/// Where keeping the copy broke off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageStep {
    Read,
    CreateFolder,
    Write,
}

impl PartialEq for NewsError {
    fn eq(&self, other: &Self) -> bool {
        let kinds = match (self, other) {
            (
                Self::Storage { step, error: _ },
                Self::Storage {
                    step: theirs,
                    error: _,
                },
            ) => step == theirs,
            (ours, theirs) => std::mem::discriminant(ours) == std::mem::discriminant(theirs),
        };

        kinds && self.to_string() == other.to_string()
    }
}

impl Eq for NewsError {}
