//! How well a row fits a search, so a list that shows only its best rows
//! shows the closest ones.

#[cfg(test)]
mod tests;

use super::{Needle, folded};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rank {
    Exact,
    Prefix,
    InName,
    /// Found by a description, keyword or other text, not by the name.
    InText,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ranked<T> {
    pub best: Rank,
    pub hits: Vec<T>,
    pub total: usize,
}

impl Needle {
    /// `None` for an empty search, which ranks nothing.
    pub fn rank<'text>(
        &self,
        name: &str,
        texts: impl IntoIterator<Item = &'text str>,
    ) -> Option<Rank> {
        if self.is_empty() {
            return None;
        }

        name_rank(self.as_str(), &folded(name)).or_else(|| {
            texts
                .into_iter()
                .any(|text| self.is_in(text))
                .then_some(Rank::InText)
        })
    }

    /// For texts folded once with [`folded`], the other texts in one string.
    pub fn rank_folded(&self, name: &str, text: &str) -> Option<Rank> {
        if self.is_empty() {
            return None;
        }

        name_rank(self.as_str(), name)
            .or_else(|| text.contains(self.as_str()).then_some(Rank::InText))
    }
}

impl<T> Ranked<T> {
    /// Lets a long list make rows only for the hits it shows.
    pub fn map<U>(self, each: impl FnMut(T) -> U) -> Ranked<U> {
        Ranked {
            best: self.best,
            hits: self.hits.into_iter().map(each).collect(),
            total: self.total,
        }
    }
}

fn name_rank(needle: &str, name: &str) -> Option<Rank> {
    if name == needle {
        Some(Rank::Exact)
    } else if name.starts_with(needle) {
        Some(Rank::Prefix)
    } else {
        name.contains(needle).then_some(Rank::InName)
    }
}

/// The best `cap` of `hits`, the ones of a rank in the order given, and
/// how many hits there are. `None` without a hit.
pub fn top<T>(hits: impl IntoIterator<Item = (Rank, T)>, cap: usize) -> Option<Ranked<T>> {
    let mut hits: Vec<(Rank, T)> = hits.into_iter().collect();
    hits.sort_by_key(|(rank, _)| *rank);

    let total = hits.len();
    let best = hits.first()?.0;
    let hits = hits.into_iter().take(cap).map(|(_, hit)| hit).collect();

    Some(Ranked { best, hits, total })
}

/// Orders `sections` by their best hit. Sections that tie keep the order
/// given.
pub fn by_best<S>(sections: &mut [S], best: impl Fn(&S) -> Rank) {
    sections.sort_by_key(|section| best(section));
}
