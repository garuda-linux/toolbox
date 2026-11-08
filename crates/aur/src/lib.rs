//! The AUR, read from its daily metadata dump only: no RPC call, no URL
//! built from input. [`Cache`] downloads the dump at most once a day with
//! a conditional request, keeps a trimmed copy in the cache folder and
//! answers from it. Every text of the dump is cleaned before it is kept.
//!
//! HTTP sits behind [`Transport`] and time behind [`Clock`], so the rules
//! are tested without a network. [`Https`] is the real transport.

mod cache;
mod clean;
mod dump;
mod https;
mod transport;

use std::time::Duration;

pub use cache::Cache;
pub use https::Https;
pub use toolbox_https::{Clock, RequestError, SystemClock};
use toolbox_plan::PackageName;
use toolbox_state::aur::AurIndex;
pub use transport::{Answer, Transport, Validators};

/// The app asks the AUR for nothing else.
pub const DUMP_URL: &str = "https://aur.archlinux.org/packages-meta-ext-v1.json.gz";

pub const PACKAGES_PAGE: &str = "https://aur.archlinux.org/packages/";

/// How long a download attempt, failed or not, blocks the next one. The
/// AUR builds the dump once a day, and a failing client must not ask
/// again and again.
pub const REFRESH_EVERY: Duration = Duration::from_hours(24);

pub trait AurSource: Send + Sync {
    /// Blocks while it downloads, so call it off the frame.
    ///
    /// # Errors
    ///
    /// [`AurError`] when there is no copy to answer with.
    fn index(&self) -> Result<AurIndex, AurError>;

    /// A caller that must answer at once waits only the first time.
    ///
    /// # Errors
    ///
    /// [`AurError`] when there is no copy to answer with.
    fn stored(&self) -> Result<AurIndex, AurError> {
        self.index()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum AurError {
    /// No copy yet and the download failed now. Words for the log and a
    /// banner's details.
    #[error("the AUR's package list could not be downloaded: {0}")]
    Download(String),
    /// No copy yet, and the last attempt was less than a day ago.
    #[error("the AUR's package list is not downloaded again before {next}")]
    Waiting {
        /// Seconds since the epoch when the next attempt is allowed.
        next: i64,
    },
    /// The cached copy or the cache folder could not be read or written.
    #[error("the AUR's package list could not be stored: {0}")]
    Storage(String),
}

/// `https://aur.archlinux.org/packages/<name>`. A [`PackageName`] holds
/// no `/`, `?` or `#`, so the link stays on that page.
pub fn package_page(name: &PackageName) -> String {
    format!("{PACKAGES_PAGE}{name}")
}
