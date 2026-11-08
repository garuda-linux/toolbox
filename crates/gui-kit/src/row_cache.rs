//! A page's filtered and sorted rows, kept until what they come from
//! changes (spotifast's table view cache).
//!
//! What they come from: the search and the other choices of the page, the
//! identity of the data, the queue's revision and the look. Row texts
//! built with Fluent arguments live in the rows, so a frame that changes
//! nothing builds no text.

#[cfg(test)]
mod tests;

use std::sync::{Arc, Weak};

/// The rows of the last inputs. `R` is every input but the texts, such
/// as the order and [`Same`] identities.
#[derive(Debug)]
pub struct RowCache<R, V> {
    kept: Option<Kept<R, V>>,
    builds: u64,
}

#[derive(Debug)]
struct Kept<R, V> {
    texts: Vec<String>,
    rest: R,
    rows: Arc<V>,
}

impl<R, V> Default for RowCache<R, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R, V> RowCache<R, V> {
    pub const fn new() -> Self {
        Self {
            kept: None,
            builds: 0,
        }
    }
}

impl<R: PartialEq, V> RowCache<R, V> {
    /// The rows for `texts` (such as the search) and `rest`, built by
    /// `build` when either differs from the last call. Equal inputs cost
    /// no allocation.
    pub fn get(&mut self, texts: &[&str], rest: R, build: impl FnOnce() -> V) -> Arc<V> {
        if let Some(kept) = &self.kept
            && kept.rest == rest
            && kept.texts.len() == texts.len()
            && kept
                .texts
                .iter()
                .zip(texts)
                .all(|(kept, text)| kept == text)
        {
            return Arc::clone(&kept.rows);
        }

        let rows = Arc::new(build());
        self.builds = self.builds.saturating_add(1);
        self.kept = Some(Kept {
            texts: texts.iter().map(|text| (*text).to_owned()).collect(),
            rest,
            rows: Arc::clone(&rows),
        });

        rows
    }

    /// How often the rows were built, for tests.
    pub const fn builds(&self) -> u64 {
        self.builds
    }
}

/// The identity of a shared value.
///
/// It is equal to another while both name the same allocation. It keeps
/// no value alive, and the allocation it names is not reused while it
/// exists, so a new value never passes for an old.
#[derive(Debug)]
pub struct Same<T>(Weak<T>);

impl<T> Same<T> {
    pub fn of(value: &Arc<T>) -> Self {
        Self(Arc::downgrade(value))
    }
}

impl<T> Clone for Same<T> {
    fn clone(&self) -> Self {
        Self(Weak::clone(&self.0))
    }
}

impl<T> PartialEq for Same<T> {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}
