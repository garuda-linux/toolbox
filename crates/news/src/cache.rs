//! The local copy of the posts and the six-hour rule.
//!
//! The time of an attempt is stored before the request goes out, so a
//! request that fails, hangs or crashes the app still counts, and the next
//! one waits its full time. The lock covers the check and the download, so
//! two readers at once never ask twice.

#[cfg(test)]
mod tests;

use std::io::{self, ErrorKind};
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use toolbox_fs::{Permissions, Replace};

use crate::feed::parse;
use crate::https::Https;
use crate::{Clock, News, NewsError, NewsSource, REFRESH_EVERY, Source, SystemClock, Transport};

/// Only the user reads the copy, as the rest of the cache folder.
const PRIVATE: Permissions = Permissions::Exactly(0o600);

/// The posts of one feed in a file of their own, downloaded at most every
/// [`REFRESH_EVERY`].
pub struct Cache {
    folder: PathBuf,
    source: Source,
    transport: Arc<dyn Transport>,
    clock: Arc<dyn Clock>,
    lock: Mutex<()>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct Stored {
    /// Seconds since the epoch.
    last_attempt: Option<i64>,
    news: Option<News>,
}

impl Cache {
    #[must_use = "the cache does nothing until asked"]
    pub fn new(
        folder: PathBuf,
        source: Source,
        transport: Arc<dyn Transport>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            folder,
            source,
            transport,
            clock,
            lock: Mutex::new(()),
        }
    }

    #[must_use = "the cache does nothing until asked"]
    pub fn system(folder: PathBuf, source: Source) -> Self {
        Self::new(
            folder,
            source,
            Arc::new(Https::new()),
            Arc::new(SystemClock),
        )
    }

    /// Downloads when the last attempt is old enough. Returns why the
    /// download failed, `None` when none was due or it worked.
    fn refresh(&self, stored: &mut Stored, now: i64) -> Result<Option<String>, NewsError> {
        // A clock set wrong forward once would block every download until
        // it passed that time. Counting the attempt as now waits from the
        // present, never longer.
        if stored.last_attempt.is_some_and(|last| last > now) {
            stored.last_attempt = Some(now);
            self.write(stored)?;
            return Ok(None);
        }

        if stored
            .last_attempt
            .is_some_and(|last| now < next_attempt(last))
        {
            return Ok(None);
        }

        // Stored first: an attempt that cannot be stored is not made, else
        // nothing would hold the next one back.
        stored.last_attempt = Some(now);
        self.write(stored)?;

        let body = match self.transport.get(self.source.feed()) {
            Ok(body) => body,
            Err(error) => return Ok(Some(error.to_string())),
        };

        let posts = match parse(&body, self.source) {
            Ok(posts) => posts,
            Err(error) => return Ok(Some(error.to_string())),
        };

        stored.news = Some(News {
            posts,
            downloaded: now,
        });
        self.write(stored)?;
        Ok(None)
    }

    /// The feed's own file, so one feed's state never touches the other's.
    fn file(&self) -> PathBuf {
        self.folder.join(format!("{}.json", self.source.key()))
    }

    /// A missing or broken file is no attempt and no copy yet. It is only
    /// ever replaced whole, so a broken one is rare.
    fn read(&self) -> Result<Stored, NewsError> {
        let file = self.file();
        match std::fs::read(&file) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes).unwrap_or_default()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Stored::default()),
            Err(error) => Err(storage(&error)),
        }
    }

    fn write(&self, stored: &Stored) -> Result<(), NewsError> {
        std::fs::create_dir_all(&self.folder).map_err(|error| storage(&error))?;
        let bytes = serde_json::to_vec(stored).map_err(|error| storage(&error))?;
        toolbox_fs::replace(&self.file(), &bytes, &Replace::new(PRIVATE))
            .map(drop)
            .map_err(|error: io::Error| storage(&error))
    }
}

impl NewsSource for Cache {
    fn news(&self) -> Result<News, NewsError> {
        let _one = self.lock.lock();
        let now = self.clock.now();
        let mut stored = self.read()?;
        let failure = self.refresh(&mut stored, now)?;

        stored.news.ok_or_else(|| {
            failure.map_or_else(
                || NewsError::Waiting {
                    next: stored.last_attempt.map_or(now, next_attempt),
                },
                NewsError::Download,
            )
        })
    }
}

fn next_attempt(last: i64) -> i64 {
    let wait = i64::try_from(REFRESH_EVERY.as_secs()).unwrap_or(i64::MAX);
    last.saturating_add(wait)
}

fn storage(error: &dyn std::fmt::Display) -> NewsError {
    NewsError::Storage(error.to_string())
}
