//! Where the app keeps its files, after the XDG base directory spec.

#[cfg(test)]
mod tests;

use std::ffi::OsString;
use std::path::{Path, PathBuf};

const APP_DIR: &str = "garuda-toolbox";

/// The app's folders. Built from an environment lookup, so tests never
/// depend on the real one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paths {
    home: PathBuf,
    config: PathBuf,
    state: PathBuf,
    cache: PathBuf,
    /// The environment named the state folder or a home, so `state` is
    /// not the `/root` fallback.
    state_named: bool,
}

impl Paths {
    #[must_use = "the paths are only computed, nothing is created"]
    pub fn from_process() -> Self {
        Self::from_env(|key| std::env::var_os(key))
    }

    /// `lookup` answers environment variables. Without an absolute `HOME`
    /// the folders fall back to `/root`, the only user without one in
    /// practice.
    #[must_use = "the paths are only computed, nothing is created"]
    pub fn from_env<F>(lookup: F) -> Self
    where
        F: Fn(&str) -> Option<OsString>,
    {
        let named_home = lookup("HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute());

        let home_named = named_home.is_some();
        let home = named_home.unwrap_or_else(|| PathBuf::from("/root"));
        let named = |variable: &str| {
            lookup(variable)
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
        };

        let base =
            |variable: &str, fallback: &str| named(variable).unwrap_or_else(|| home.join(fallback));

        let named_state = named("XDG_STATE_HOME");

        Self {
            config: base("XDG_CONFIG_HOME", ".config"),
            state_named: home_named || named_state.is_some(),
            state: named_state.unwrap_or_else(|| home.join(".local/state")),
            cache: base("XDG_CACHE_HOME", ".cache"),
            home,
        }
    }

    #[must_use = "the path is only computed, nothing is read"]
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// The app's state folder when the environment names a home or a state
    /// folder. `None` when it would only be the `/root` fallback, which is
    /// no place for a user's files.
    #[must_use = "the path is only computed, nothing is created"]
    pub fn own_state_dir(&self) -> Option<PathBuf> {
        self.state_named.then(|| self.state_dir())
    }

    #[must_use = "the path is only computed, nothing is created"]
    pub fn settings_file(&self) -> PathBuf {
        self.config.join(APP_DIR).join("settings.toml")
    }

    fn state_dir(&self) -> PathBuf {
        self.state.join(APP_DIR)
    }

    #[must_use = "the path is only computed, nothing is created"]
    pub fn staged_changes_file(&self) -> PathBuf {
        self.state_dir().join("changes.json")
    }

    #[must_use = "the path is only computed, nothing is created"]
    pub fn crash_file(&self) -> PathBuf {
        self.state_dir().join("crash.log")
    }

    #[must_use = "the path is only computed, nothing is created"]
    pub fn cache_dir(&self) -> PathBuf {
        self.cache.join(APP_DIR)
    }

    #[must_use = "the path is only computed, nothing is created"]
    pub fn aur_dir(&self) -> PathBuf {
        self.cache_dir().join("aur")
    }

    #[must_use = "the path is only computed, nothing is created"]
    pub fn news_dir(&self) -> PathBuf {
        self.cache_dir().join("news")
    }

    #[must_use = "the path is only computed, nothing is created"]
    pub fn autostart_dir(&self) -> PathBuf {
        self.config.join("autostart")
    }
}
