//! The guide as the app shows it: the pages in the summary's order, each
//! with its blocks and headings, and the search over them.

#[cfg(test)]
mod tests;

use std::ops::Range;

use crate::markdown::{self, Block};
use crate::search::{self, Hit, Index};
use crate::summary;

#[derive(Debug)]
pub struct Guide {
    parts: Vec<GuidePart>,
    pages: Vec<Page>,
    /// Names the summary lists without a file, for the tests that guard
    /// the summary.
    #[cfg(test)]
    missing: Vec<String>,
    index: Index,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuidePart {
    pub title: Option<String>,
    /// The part's pages, as positions in [`Guide::pages`].
    pub pages: Range<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Page {
    /// The file under `docs/src`, which also names the page in links.
    pub file: String,
    pub title: String,
    pub part: Option<String>,
    pub blocks: Vec<Block>,
    /// The headings below the page's title, for "On this page".
    pub outline: Vec<Outline>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outline {
    pub level: u8,
    pub anchor: String,
    pub text: String,
}

impl Guide {
    /// The guide of `summary` and the pages in `sources` as file and
    /// markdown. A file the summary does not name is left out, and so is a
    /// name without its file.
    pub fn new(summary: &str, sources: &[(&str, &str)]) -> Self {
        let mut parts = Vec::new();
        let mut pages = Vec::new();
        #[cfg(test)]
        let mut missing = Vec::new();

        for part in summary::parse(summary) {
            let first = pages.len();
            for entry in part.entries {
                let Some((_, source)) = sources.iter().find(|(file, _)| *file == entry.file) else {
                    #[cfg(test)]
                    missing.push(entry.file);
                    continue;
                };

                pages.push(page(entry, part.title.clone(), source));
            }

            parts.push(GuidePart {
                title: part.title,
                pages: first..pages.len(),
            });
        }

        let index = Index::new(&pages);
        Self {
            parts,
            pages,
            #[cfg(test)]
            missing,
            index,
        }
    }

    pub fn parts(&self) -> &[GuidePart] {
        &self.parts
    }

    pub fn pages(&self) -> &[Page] {
        &self.pages
    }

    #[cfg(test)]
    pub fn missing(&self) -> &[String] {
        &self.missing
    }

    pub fn page(&self, file: &str) -> Option<&Page> {
        self.pages.iter().find(|page| page.file == file)
    }

    pub fn previous(&self, file: &str) -> Option<&Page> {
        let before = self.position(file)?.checked_sub(1)?;
        self.pages.get(before)
    }

    pub fn next(&self, file: &str) -> Option<&Page> {
        self.pages.get(self.position(file)? + 1)
    }

    /// The sections that hold every word of `query`, page titles first,
    /// then headings, then the text, each in reading order.
    pub fn search(&self, query: &str) -> Vec<Hit> {
        search::find(&self.index, &self.pages, query)
    }

    fn position(&self, file: &str) -> Option<usize> {
        self.pages.iter().position(|page| page.file == file)
    }
}

fn page(entry: summary::Entry, part: Option<String>, source: &str) -> Page {
    let blocks = markdown::parse(source);
    let outline = blocks
        .iter()
        .filter_map(|block| {
            let heading = block.heading().filter(|heading| heading.level > 1)?;
            Some(Outline {
                level: heading.level,
                anchor: heading.anchor.clone(),
                text: block.plain_text(),
            })
        })
        .collect();

    Page {
        file: entry.file,
        title: entry.title,
        part,
        blocks,
        outline,
    }
}
