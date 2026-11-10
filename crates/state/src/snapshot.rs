//! What the system looks like right now, as far as changes can touch it.
//!
//! Each part of the state owns its own entry, stored by type, so a new
//! part arrives without touching this file. Probes deliver whole parts.

#[cfg(test)]
mod tests;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// A type a probe delivers whole. The marker keeps any other value, such
/// as a bare `String` that two probes could both mean, out of the store.
pub trait Part: Send + Sync + 'static {}

/// The state of a domain that needs none.
impl Part for () {}

/// A copy shares its parts with the original, so the store can add a part
/// no probe reads (the lock flags) to a snapshot the pages share already.
#[derive(Clone, Default)]
pub struct Snapshot {
    parts: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Snapshot {
    pub fn insert<T>(&mut self, part: T)
    where
        T: Part,
    {
        self.parts.insert(TypeId::of::<T>(), Arc::new(part));
    }

    /// The part of type `T`, or `None` while no probe has delivered it.
    pub fn part<T>(&self) -> Option<&T>
    where
        T: Part,
    {
        self.parts
            .get(&TypeId::of::<T>())
            .and_then(|part| part.as_ref().downcast_ref::<T>())
    }
}

impl fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Snapshot")
            .field("parts", &self.parts.len())
            .finish()
    }
}
