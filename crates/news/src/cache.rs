//! The local copy of the posts and the six-hour rule. The lock covers the
//! check and the download, so two readers of this process at once never
//! ask twice.

#[cfg(test)]
mod tests;

use std::io::{self, ErrorKind};
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use toolbox_fs::{Permissions, Replace};
use toolbox_https::{Attempt, due, next_attempt};

use crate::feed::parse;
use crate::https::Https;
use crate::{
    Clock, News, NewsError, NewsSource, REFRESH_EVERY, Source, StorageStep, SystemClock, Transport,
};

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
            Arc::new(Https::new(source)),
            Arc::new(SystemClock),
        )
    }

    /// Downloads when the attempt is due. Returns why the download failed,
    /// `None` when none was due or it worked.
    fn refresh(&self, stored: &mut Stored, now: i64) -> Result<Option<NewsError>, NewsError> {
        match due(stored.last_attempt, now, REFRESH_EVERY) {
            Attempt::ClockAhead => {
                stored.last_attempt = Some(now);
                self.write(stored)?;
                return Ok(None);
            }
            Attempt::NotDue => return Ok(None),
            Attempt::Due => {}
        }

        stored.last_attempt = Some(now);
        self.write(stored)?;

        let body = match self.transport.get(self.source.feed()) {
            Ok(body) => body,
            Err(error) => return Ok(Some(NewsError::Request(error))),
        };

        let posts = match parse(&body, self.source) {
            Ok(posts) => posts,
            Err(error) => return Ok(Some(NewsError::Feed(error))),
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
            Err(error) => Err(storage(StorageStep::Read)(error)),
        }
    }

    fn write(&self, stored: &Stored) -> Result<(), NewsError> {
        std::fs::create_dir_all(&self.folder).map_err(storage(StorageStep::CreateFolder))?;
        let bytes =
            serde_json::to_vec(stored).map_err(|error| NewsError::Encode(Arc::new(error)))?;

        toolbox_fs::replace(&self.file(), &bytes, &Replace::new(PRIVATE))
            .map(drop)
            .map_err(storage(StorageStep::Write))
    }
}

impl NewsSource for Cache {
    fn news(&self) -> Result<News, NewsError> {
        let _one = self.lock.lock();
        let now = self.clock.now();
        let mut stored = self.read()?;
        let failure = self.refresh(&mut stored, now)?;

        if let Some(why) = &failure {
            tracing::warn!("the news were not refreshed: {why}");
        }

        stored.news.ok_or_else(|| {
            failure.unwrap_or_else(|| NewsError::Waiting {
                next: stored
                    .last_attempt
                    .map_or(now, |last| next_attempt(last, REFRESH_EVERY)),
            })
        })
    }
}

fn storage(step: StorageStep) -> impl Fn(io::Error) -> NewsError {
    move |error| NewsError::Storage {
        step,
        error: Arc::new(error),
    }
}
