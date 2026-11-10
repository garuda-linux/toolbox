//! The queue's own `want` and `remove`. Outside `toolbox-core` the pages
//! and commands queue through the features, so only tests reach these.

use crate::{ChangeSet, Domain, Snapshot};

pub trait Queue {
    fn want<D>(&mut self, item: D::Item, wish: D::Wish, snapshot: &Snapshot)
    where
        D: Domain;

    fn remove<D>(&mut self, item: &D::Item)
    where
        D: Domain;
}

impl Queue for ChangeSet {
    fn want<D>(&mut self, item: D::Item, wish: D::Wish, snapshot: &Snapshot)
    where
        D: Domain,
    {
        Self::want::<D>(self, item, wish, snapshot);
    }

    fn remove<D>(&mut self, item: &D::Item)
    where
        D: Domain,
    {
        Self::remove::<D>(self, item);
    }
}
