//! Full-text search over the guide. Each section, from one heading to the
//! next, is one place to find. The text is lowered once when the guide is
//! built, so a search lowers only the query.

use crate::guide::Page;

pub const MAX_HITS: usize = 50;

/// How many characters of the text show before the first word found.
const BEFORE: usize = 40;
/// How many characters a snippet holds at most.
const SNIPPET: usize = 140;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    pub file: String,
    pub page_title: String,
    /// The section's heading, `None` for the top of the page.
    pub heading: Option<String>,
    pub anchor: Option<String>,
    pub snippet: String,
}

#[derive(Debug, Default)]
pub struct Index {
    sections: Vec<Section>,
}

#[derive(Debug)]
struct Section {
    page: usize,
    /// The anchor and words of the heading, `None` for the top of a page.
    heading: Option<(String, String)>,
    text: String,
    /// The page title for the top of a page, else empty, lowered.
    title_lower: String,
    heading_lower: String,
    text_lower: String,
}

impl Index {
    pub fn new(pages: &[Page]) -> Self {
        let mut sections = Vec::new();
        for (at, page) in pages.iter().enumerate() {
            let mut current = Section::top(at, &page.title);
            let mut words = Vec::new();
            for block in &page.blocks {
                match block.heading() {
                    Some(heading) if heading.level == 1 => {}
                    Some(heading) => {
                        sections.push(current.with_text(&words));
                        words.clear();
                        current = Section::under(at, &heading.anchor, &block.plain_text());
                    }
                    None => words.push(block.plain_text()),
                }
            }

            sections.push(current.with_text(&words));
        }

        Self { sections }
    }
}

impl Section {
    fn top(page: usize, title: &str) -> Self {
        Self {
            page,
            heading: None,
            text: String::new(),
            title_lower: title.to_lowercase(),
            heading_lower: String::new(),
            text_lower: String::new(),
        }
    }

    fn under(page: usize, anchor: &str, heading: &str) -> Self {
        Self {
            heading_lower: heading.to_lowercase(),
            heading: Some((anchor.to_owned(), heading.to_owned())),
            ..Self::top(page, "")
        }
    }

    fn with_text(mut self, words: &[String]) -> Self {
        self.text = words.join(" ");
        self.text_lower = self.text.to_lowercase();
        self
    }

    /// Where every term shows: 0 the title, 1 the heading, 2 anywhere in
    /// the section. `None` when a term is missing.
    fn rank(&self, terms: &[String]) -> Option<u8> {
        let all = |text: &str| terms.iter().all(|term| text.contains(term.as_str()));
        if !self.title_lower.is_empty() && all(&self.title_lower) {
            return Some(0);
        }

        if all(&self.heading_lower) {
            return Some(1);
        }

        terms
            .iter()
            .all(|term| {
                self.title_lower.contains(term.as_str())
                    || self.heading_lower.contains(term.as_str())
                    || self.text_lower.contains(term.as_str())
            })
            .then_some(2)
    }

    /// Up to [`SNIPPET`] characters of the text, from a little before the
    /// first term found.
    fn snippet(&self, terms: &[String]) -> String {
        // Lowering keeps the byte places of ASCII text only. Other text
        // shows from its start.
        let found = terms
            .iter()
            .filter_map(|term| self.text_lower.find(term.as_str()))
            .min()
            .filter(|_| self.text_lower.len() == self.text.len())
            .unwrap_or(0);

        let chars_before = self
            .text
            .char_indices()
            .take_while(|(at, _)| *at < found)
            .count();

        let skip = chars_before.saturating_sub(BEFORE);
        let total = self.text.chars().count();

        let piece: String = self.text.chars().skip(skip).take(SNIPPET).collect();

        let mut snippet = String::new();
        if skip > 0 {
            snippet.push('…');
        }

        snippet.push_str(piece.trim());
        if skip + SNIPPET < total {
            snippet.push('…');
        }

        snippet
    }
}

pub fn find(index: &Index, pages: &[Page], query: &str) -> Vec<Hit> {
    let terms: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    if terms.is_empty() {
        return Vec::new();
    }

    let mut ranked: Vec<(u8, usize, &Section)> = index
        .sections
        .iter()
        .enumerate()
        .filter_map(|(order, section)| Some((section.rank(&terms)?, order, section)))
        .collect();

    ranked.sort_by_key(|&(rank, order, _)| (rank, order));

    ranked
        .into_iter()
        .take(MAX_HITS)
        .filter_map(|(_, _, section)| {
            let page = pages.get(section.page)?;
            Some(Hit {
                file: page.file.clone(),
                page_title: page.title.clone(),
                heading: section.heading.as_ref().map(|(_, text)| text.clone()),
                anchor: section.heading.as_ref().map(|(anchor, _)| anchor.clone()),
                snippet: section.snippet(&terms),
            })
        })
        .collect()
}
