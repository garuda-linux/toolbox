//! The local copy of the dump and the once-a-day rule.
//!
//! The time of an attempt is stored before the request goes out, so a
//! request that fails, hangs or crashes the app still counts, and the
//! next one waits a full day. The lock covers the check and the download,
//! so two readers at once never ask twice.

#[cfg(test)]
mod tests;

use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use toolbox_fs::{Permissions, Replace};
use toolbox_https::{Clock, SystemClock};
use toolbox_state::aur::AurIndex;

use crate::dump::parse;
use crate::https::Https;
use crate::transport::{Answer, Transport, Validators};
use crate::{AurError, AurSource, DUMP_URL, REFRESH_EVERY};

const INDEX_FILE: &str = "index.json";
const META_FILE: &str = "meta.json";

/// The dump unpacks to about 80 MB. Far more is no dump of the AUR.
const MAX_UNPACKED_BYTES: u64 = 512 << 20;

/// Only the user reads the copy, as the rest of the cache folder.
const PRIVATE: Permissions = Permissions::Exactly(0o600);

/// The trimmed copy in a folder of its own, downloaded at most once a
/// day.
pub struct Cache {
    folder: PathBuf,
    transport: Arc<dyn Transport>,
    clock: Arc<dyn Clock>,
    lock: Mutex<()>,
}

/// What the last attempt left behind besides the copy.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct Meta {
    #[serde(flatten)]
    validators: Validators,
    /// Seconds since the epoch.
    last_attempt: Option<i64>,
}

impl Cache {
    #[must_use = "the cache does nothing until asked"]
    pub fn new(folder: PathBuf, transport: Arc<dyn Transport>, clock: Arc<dyn Clock>) -> Self {
        Self {
            folder,
            transport,
            clock,
            lock: Mutex::new(()),
        }
    }

    /// The copy in `folder`, downloaded over HTTPS by the system's clock.
    #[must_use = "the cache does nothing until asked"]
    pub fn system(folder: PathBuf) -> Self {
        Self::new(folder, Arc::new(Https::new()), Arc::new(SystemClock))
    }

    /// When the copy was last written, in seconds since the epoch, `None`
    /// without a copy. It tells a changed copy apart without reading it.
    pub fn modified(&self) -> Option<u64> {
        std::fs::metadata(self.folder.join(INDEX_FILE))
            .and_then(|meta| meta.modified())
            .ok()?
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|since| since.as_secs())
    }

    /// The copy after a refresh when one is due. The caller holds the lock.
    fn refreshed(&self) -> Result<AurIndex, AurError> {
        let now = self.clock.now();
        let failure = self.refresh(now)?;

        self.copy()?.ok_or_else(|| {
            failure.map_or_else(
                || AurError::Waiting {
                    next: self.meta().last_attempt.map_or(now, next_attempt),
                },
                AurError::Download,
            )
        })
    }

    /// Downloads when the last attempt is a day old. Returns why the
    /// download failed, `None` when none was due or it worked.
    fn refresh(&self, now: i64) -> Result<Option<String>, AurError> {
        let mut meta = self.meta();

        // A clock set wrong forward once would block every download until
        // it passed that time. Counting the attempt as now waits one day
        // from the present, never longer.
        if meta.last_attempt.is_some_and(|last| last > now) {
            meta.last_attempt = Some(now);
            self.write(META_FILE, &meta)?;
            return Ok(None);
        }

        if meta.last_attempt.is_some_and(|last| !is_due(last, now)) {
            return Ok(None);
        }

        // Stored first: an attempt that cannot be stored is not made, else
        // nothing would hold the next one back.
        meta.last_attempt = Some(now);
        self.write(META_FILE, &meta)?;

        // Without a copy a "not modified" would leave nothing to show.
        let has_copy = self.folder.join(INDEX_FILE).is_file();
        let sent = if has_copy {
            meta.validators.clone()
        } else {
            Validators::default()
        };

        let answer = match self.transport.get(DUMP_URL, &sent) {
            Ok(answer) => answer,
            Err(error) => return Ok(Some(error.to_string())),
        };

        let Answer::Fresh { body, validators } = answer else {
            return Ok(None);
        };

        let packages = match parse(body, MAX_UNPACKED_BYTES) {
            Ok(packages) => packages,
            Err(error) => return Ok(Some(error.to_string())),
        };

        self.write(
            INDEX_FILE,
            &AurIndex {
                packages,
                downloaded: now,
            },
        )?;

        meta.validators = validators;
        self.write(META_FILE, &meta)?;
        Ok(None)
    }

    /// A missing or broken meta file is no attempt yet. Both files are
    /// only ever replaced whole, so a broken one is rare.
    fn meta(&self) -> Meta {
        std::fs::read(self.folder.join(META_FILE))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    fn copy(&self) -> Result<Option<AurIndex>, AurError> {
        let file = self.folder.join(INDEX_FILE);
        match std::fs::read(&file) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|error| storage(&file, &error)),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(storage(&file, &error)),
        }
    }

    fn write(&self, name: &str, value: &impl Serialize) -> Result<(), AurError> {
        let file = self.folder.join(name);
        let bytes = serde_json::to_vec(value).map_err(|error| storage(&file, &error))?;
        toolbox_fs::replace(&file, &bytes, &Replace::new(PRIVATE))
            .map(drop)
            .map_err(|error: io::Error| storage(&file, &error))
    }
}

impl AurSource for Cache {
    fn index(&self) -> Result<AurIndex, AurError> {
        let _one = self.lock.lock();
        self.refreshed()
    }

    fn stored(&self) -> Result<AurIndex, AurError> {
        let _one = self.lock.lock();
        self.copy()?.map_or_else(|| self.refreshed(), Ok)
    }
}

fn is_due(last: i64, now: i64) -> bool {
    now >= next_attempt(last)
}

fn next_attempt(last: i64) -> i64 {
    let day = i64::try_from(REFRESH_EVERY.as_secs()).unwrap_or(i64::MAX);
    last.saturating_add(day)
}

fn storage(file: &Path, error: &dyn std::fmt::Display) -> AurError {
    AurError::Storage(format!("{}: {error}", file.display()))
}
