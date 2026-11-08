//! A sorted, duplicate-free and never empty list of targets, so an action
//! like "install nothing" cannot exist.

#[cfg(test)]
mod tests;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("an action needs at least one target")]
pub struct EmptyTargetList;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetList<T> {
    targets: Vec<T>,
}

impl<T> TargetList<T>
where
    T: Ord,
{
    /// # Errors
    ///
    /// [`EmptyTargetList`] when `targets` yields nothing.
    pub fn new(targets: impl IntoIterator<Item = T>) -> Result<Self, EmptyTargetList> {
        let mut targets: Vec<T> = targets.into_iter().collect();
        if targets.is_empty() {
            return Err(EmptyTargetList);
        }

        targets.sort();
        targets.dedup();
        Ok(Self { targets })
    }
}

impl<T> TargetList<T> {
    /// A list of one, which cannot be empty.
    pub fn one(target: T) -> Self {
        Self {
            targets: vec![target],
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.targets.iter()
    }

    pub const fn len(&self) -> usize {
        self.targets.len()
    }

    /// Always `false`. Exists because clippy wants it beside `len`.
    pub const fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }
}

impl<'list, T> IntoIterator for &'list TargetList<T> {
    type IntoIter = std::slice::Iter<'list, T>;
    type Item = &'list T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> Serialize for TargetList<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.targets.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for TargetList<T>
where
    T: Deserialize<'de> + Ord,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let targets = Vec::<T>::deserialize(deserializer)?;
        Self::new(targets).map_err(serde::de::Error::custom)
    }
}
