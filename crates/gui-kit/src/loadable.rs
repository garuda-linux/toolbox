//! A value that comes from background work, after spotifast's `Loadable`.
//!
//! Plain state without a channel: the service that loads the value sets
//! it, the page reads it every frame. The last good value stays while a
//! new load runs or after one fails, so nothing flickers.

#[cfg(test)]
mod tests;

use std::sync::Arc;

/// Where a background value stands. `E` says why a load failed: a reason
/// for the log by default, or a type the page words.
#[derive(Debug, PartialEq, Eq)]
pub enum Loadable<T, E = String> {
    NotLoaded,
    Loading { previous: Option<Arc<T>> },
    Loaded(Arc<T>),
    Failed { error: E, previous: Option<Arc<T>> },
}

/// Written out, as the derive would ask `T` and `E` for a default too.
impl<T, E> Default for Loadable<T, E> {
    fn default() -> Self {
        Self::NotLoaded
    }
}

/// Only the [`Arc`] is copied, so `T` needs no `Clone`.
impl<T, E: Clone> Clone for Loadable<T, E> {
    fn clone(&self) -> Self {
        match self {
            Self::NotLoaded => Self::NotLoaded,
            Self::Loading { previous } => Self::Loading {
                previous: previous.clone(),
            },
            Self::Loaded(value) => Self::Loaded(Arc::clone(value)),
            Self::Failed { error, previous } => Self::Failed {
                error: error.clone(),
                previous: previous.clone(),
            },
        }
    }
}

impl<T, E> Loadable<T, E> {
    /// The newest value there is, also while loading or after a failure.
    pub fn get(&self) -> Option<&T> {
        self.latest().map(Arc::as_ref)
    }

    /// [`Loadable::get`] as the shared value, to tell a new one from the
    /// last.
    pub const fn latest(&self) -> Option<&Arc<T>> {
        match self {
            Self::NotLoaded => None,
            Self::Loaded(value) => Some(value),
            Self::Loading { previous } | Self::Failed { error: _, previous } => previous.as_ref(),
        }
    }

    /// The error of the last load, until the next one starts.
    pub const fn error(&self) -> Option<&E> {
        match self {
            Self::Failed { error, previous: _ } => Some(error),
            Self::NotLoaded | Self::Loading { previous: _ } | Self::Loaded(_) => None,
        }
    }

    pub const fn is_loading(&self) -> bool {
        matches!(self, Self::Loading { previous: _ })
    }

    /// A load starts. The newest value stays readable until it ends.
    pub fn start_loading(&mut self) {
        let previous = self.take_latest();
        *self = Self::Loading { previous };
    }

    /// A load ended. A failure keeps the newest value readable.
    pub fn finish(&mut self, result: Result<T, E>) {
        match result {
            Ok(value) => *self = Self::Loaded(Arc::new(value)),
            Err(error) => self.fail(error),
        }
    }

    /// A load ended with a value others share already.
    pub fn set_loaded(&mut self, value: Arc<T>) {
        *self = Self::Loaded(value);
    }

    /// A load failed. The newest value stays readable.
    pub fn fail(&mut self, error: E) {
        let previous = self.take_latest();
        *self = Self::Failed { error, previous };
    }

    fn take_latest(&mut self) -> Option<Arc<T>> {
        match std::mem::take(self) {
            Self::NotLoaded => None,
            Self::Loaded(value) => Some(value),
            Self::Loading { previous } | Self::Failed { error: _, previous } => previous,
        }
    }
}
