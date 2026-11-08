//! The search fields of the lists: whether a text holds what was typed,
//! whatever the case of either.
//!
//! Both sides are compared as `str::to_lowercase` would spell them, letter
//! by letter, so no lowered copy of every text is made per keystroke. One
//! difference: the two lower case sigmas count as one letter, where
//! `to_lowercase` picks one by the place in the word.

#[cfg(test)]
mod tests;

const FINAL_SIGMA: char = 'ς';
const SIGMA: char = 'σ';

/// What the user typed, ready to be looked for in many texts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Needle(String);

impl Needle {
    /// `search` without the blanks around it.
    pub fn new(search: &str) -> Self {
        Self(folded(search.trim()))
    }

    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The folded search, for lists that fold their texts once and keep
    /// them.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether `text` holds the search. An empty search is in every text.
    ///
    /// # Examples
    ///
    /// ```
    /// use toolbox_core::features::search::Needle;
    ///
    /// let needle = Needle::new("AC 95");
    ///
    /// assert!(needle.is_in("Wireless-AC 9560"));
    /// assert!(!needle.is_in("Ethernet"));
    /// ```
    pub fn is_in(&self, text: &str) -> bool {
        let mut rest = letters(text);

        loop {
            let mut here = rest.clone();

            if self.0.chars().all(|letter| here.next() == Some(letter)) {
                return true;
            }

            // One letter of the lowered text at a time: a match may start
            // inside a letter that is several in lower case.
            if rest.next().is_none() {
                return false;
            }
        }
    }

    /// Whether one of the row's texts holds the search. An empty search
    /// finds every row.
    pub fn finds<R>(&self, row: &R) -> bool
    where
        R: Searchable + ?Sized,
    {
        self.is_empty() || row.texts().any(|text| self.is_in(text))
    }
}

/// A row of a list with a search field: the texts the search looks in.
/// A row keeps every text it is found by, numbers too, so a search
/// formats nothing per row and keystroke.
pub trait Searchable {
    fn texts(&self) -> impl Iterator<Item = &str>;
}

pub fn filter<'rows, R, I>(rows: I, needle: &Needle) -> impl Iterator<Item = &'rows R>
where
    R: Searchable + 'rows,
    I: IntoIterator<Item = &'rows R>,
{
    rows.into_iter().filter(move |row| needle.finds(*row))
}

/// `text` lowered the way [`Needle`] compares, as one string. Discover
/// folds its apps once with this when they are loaded, so every search
/// agrees.
pub fn folded(text: &str) -> String {
    letters(text).collect()
}

fn letters(text: &str) -> impl Iterator<Item = char> + Clone {
    text.chars()
        .flat_map(char::to_lowercase)
        .map(|letter| if letter == FINAL_SIGMA { SIGMA } else { letter })
}
