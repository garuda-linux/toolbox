//! A page's markdown as blocks the app draws with its own widgets. Only
//! what the guide uses is kept: raw HTML is left out, and a link goes to
//! another page, a heading or the web, never anywhere else.

#[cfg(test)]
mod tests;

mod anchor;
mod blocks;
mod inline;

pub use anchor::slug;

/// One block of a page, in reading order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    Heading(Heading),
    Paragraph(Vec<Span>),
    List(List),
    Table(Table),
    Code(Code),
    /// A quote, or a note or warning when it starts with `**Note:**` or
    /// `**Warning:**`, which the label of the callout replaces.
    Callout(Callout),
    Image(Image),
    Rule,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Heading {
    pub level: u8,
    /// The heading's id, as mdBook makes it, unique on the page.
    pub anchor: String,
    pub spans: Vec<Span>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct List {
    /// The first number of a numbered list.
    pub start: Option<u64>,
    pub items: Vec<Vec<Block>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    pub head: Vec<Cell>,
    pub rows: Vec<Vec<Cell>>,
}

pub type Cell = Vec<Span>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Code {
    pub language: Option<String>,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Callout {
    pub kind: CalloutKind,
    pub blocks: Vec<Block>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalloutKind {
    Note,
    Warning,
    Quote,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    /// The picture's path, relative to `docs/src`.
    pub src: String,
    pub alt: String,
}

/// A run of text in one style.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub style: Style,
    pub link: Option<Link>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub strong: bool,
    pub emphasis: bool,
    pub code: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Link {
    /// Another page of the guide, at a heading or its top.
    Page {
        file: String,
        anchor: Option<String>,
    },
    /// A heading of the same page.
    Anchor(String),
    /// An `http` or `https` address, for the browser.
    Web(String),
}

impl Block {
    pub const fn heading(&self) -> Option<&Heading> {
        if let Self::Heading(heading) = self {
            Some(heading)
        } else {
            None
        }
    }

    /// The blocks inside a list or callout, every item's in turn.
    pub fn children(&self) -> Vec<&Self> {
        match self {
            Self::List(list) => list.items.iter().flatten().collect(),
            Self::Callout(callout) => callout.blocks.iter().collect(),
            Self::Heading(_)
            | Self::Paragraph(_)
            | Self::Table(_)
            | Self::Code(_)
            | Self::Image(_)
            | Self::Rule => Vec::new(),
        }
    }

    /// The text runs of a heading, paragraph or every table cell.
    pub fn spans(&self) -> Vec<&Span> {
        match self {
            Self::Heading(heading) => heading.spans.iter().collect(),
            Self::Paragraph(spans) => spans.iter().collect(),
            Self::Table(table) => table
                .head
                .iter()
                .chain(table.rows.iter().flatten())
                .flatten()
                .collect(),
            Self::List(_) | Self::Code(_) | Self::Callout(_) | Self::Image(_) | Self::Rule => {
                Vec::new()
            }
        }
    }

    /// The block's words without styles, for the search.
    pub fn plain_text(&self) -> String {
        let mut words = Vec::new();
        self.collect_text(&mut words);
        words.join(" ")
    }

    fn collect_text(&self, out: &mut Vec<String>) {
        match self {
            Self::Heading(_) | Self::Paragraph(_) => {
                out.push(self.spans().iter().map(|span| span.text.as_str()).collect());
            }
            Self::Table(table) => {
                for cell in table.head.iter().chain(table.rows.iter().flatten()) {
                    out.push(cell.iter().map(|span| span.text.as_str()).collect());
                }
            }
            Self::List(_) | Self::Callout(_) => {
                for child in self.children() {
                    child.collect_text(out);
                }
            }
            Self::Code(code) => out.push(code.text.clone()),
            Self::Image(image) => out.push(image.alt.clone()),
            Self::Rule => {}
        }
    }
}

pub fn parse(markdown: &str) -> Vec<Block> {
    blocks::parse(markdown)
}
