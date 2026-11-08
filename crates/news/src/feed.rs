//! A news feed as plain posts. Anyone with a forum account can title a
//! post, so titles never reach the display with control characters or
//! bidi overrides, nor at any length, and a link that does not lead to
//! its source's site is dropped.

#[cfg(test)]
mod tests;

use quick_xml::Reader;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::Event;
use serde::{Deserialize, Serialize};

use crate::Source;

/// The feed lists the newest first, Home shows a few of them.
pub const MAX_POSTS: usize = 10;
pub const TITLE_CHARS: usize = 200;
const LINK_CHARS: usize = 2048;
/// An item names one or two categories, more are not read.
const MAX_CATEGORIES: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Post {
    pub title: String,
    /// Only a link into its source's site, `None` for any other.
    pub link: Option<String>,
    /// Seconds since the epoch, `None` when the feed gave no valid date.
    pub published: Option<i64>,
    #[serde(default)]
    pub source: Source,
    /// The post asks for a step of the user around an update.
    #[serde(default)]
    pub manual: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the feed is no valid XML: {0}")]
pub struct FeedError(String);

#[derive(Default)]
struct Item {
    title: String,
    link: String,
    date: String,
    categories: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Field {
    Title,
    Link,
    Date,
    Category,
}

#[derive(Default)]
struct Reading {
    source: Source,
    posts: Vec<Post>,
    item: Option<Item>,
    field: Option<Field>,
    /// Elements still open, so a feed cut short is an error, not "no
    /// posts".
    open: usize,
}

impl Reading {
    fn take(&mut self, event: Event<'_>) {
        match event {
            Event::Start(start) => {
                self.open += 1;
                self.field = match start.local_name().as_ref() {
                    "item" => {
                        self.item = Some(Item::default());
                        None
                    }
                    "title" if self.item.is_some() => Some(Field::Title),
                    "link" if self.item.is_some() => Some(Field::Link),
                    "pubDate" if self.item.is_some() => Some(Field::Date),
                    "category" => self.item.as_mut().and_then(|item| {
                        (item.categories.len() < MAX_CATEGORIES).then(|| {
                            item.categories.push(String::new());
                            Field::Category
                        })
                    }),
                    _ => None,
                };
            }
            Event::End(end) => {
                self.open = self.open.saturating_sub(1);
                if end.local_name().as_ref() == "item"
                    && let Some(done) = self.item.take()
                    && let Some(post) = post(&done, self.source)
                {
                    self.posts.push(post);
                }

                self.field = None;
            }
            Event::Text(text) => self.push(&text.xml10_content()),
            Event::CData(data) => self.push(&data.xml10_content()),
            Event::GeneralRef(reference) => {
                let resolved = if reference.is_char_ref() {
                    reference
                        .resolve_char_ref()
                        .ok()
                        .flatten()
                        .map(String::from)
                } else {
                    resolve_predefined_entity(&reference).map(str::to_owned)
                };

                self.push(&resolved.unwrap_or_default());
            }
            Event::Empty(_)
            | Event::Comment(_)
            | Event::Decl(_)
            | Event::PI(_)
            | Event::DocType(_)
            | Event::Eof => {}
        }
    }

    fn push(&mut self, text: &str) {
        let (Some(item), Some(field)) = (self.item.as_mut(), self.field) else {
            return;
        };

        let target = match field {
            Field::Category => match item.categories.last_mut() {
                Some(category) => category,
                None => return,
            },
            Field::Title => &mut item.title,
            Field::Link => &mut item.link,
            Field::Date => &mut item.date,
        };

        // Bounded while reading, so a huge field never builds up.
        if target.len() < LINK_CHARS * 4 {
            target.push_str(text);
        }
    }
}

/// # Errors
///
/// [`FeedError`] when the feed is no well-formed XML.
pub fn parse(feed: &[u8], source: Source) -> Result<Vec<Post>, FeedError> {
    let mut reader = Reader::from_reader(feed);
    reader.config_mut().check_end_names = true;

    let mut reading = Reading {
        source,
        ..Reading::default()
    };

    let mut buffer = Vec::new();
    loop {
        let event = reader
            .read_event_into(&mut buffer)
            .map_err(|error| FeedError(error.to_string()))?;

        if matches!(event, Event::Eof) {
            if reading.open > 0 {
                return Err(FeedError("the feed ends early".to_owned()));
            }

            break;
        }

        reading.take(event);
        buffer.clear();
        if reading.posts.len() == MAX_POSTS {
            break;
        }
    }

    Ok(reading.posts)
}

fn post(item: &Item, source: Source) -> Option<Post> {
    let title: String = plain(item.title.trim()).take(TITLE_CHARS).collect();
    if title.trim().is_empty() {
        return None;
    }

    let link = item.link.trim();
    let link = (link.starts_with(source.post_prefix())
        && link.len() <= LINK_CHARS
        && link.chars().all(|character| character.is_ascii_graphic()))
    .then(|| link.to_owned());

    let published = jiff::fmt::rfc2822::parse(item.date.trim())
        .ok()
        .map(|zoned| zoned.timestamp().as_second());

    let manual = source.asks_for_a_step(&title, &item.categories);

    Some(Post {
        title,
        link,
        published,
        source,
        manual,
    })
}

fn plain(text: &str) -> impl Iterator<Item = char> + '_ {
    text.chars()
        .filter(|character| !toolbox_plan::text::is_hidden(*character))
}
