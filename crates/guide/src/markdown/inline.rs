//! The text inside a block: styles, code and links as spans, pictures
//! apart, raw HTML left out.

use std::iter::Peekable;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};

use super::{Image, Link, Span, Style};

/// The spans of one run of text and the pictures in it.
#[derive(Debug, Default)]
pub(super) struct Inline {
    pub spans: Vec<Span>,
    pub images: Vec<Image>,
}

/// The styles and links open at one point of the text.
#[derive(Debug, Default)]
struct Open {
    strong: u32,
    emphasis: u32,
    links: Vec<Option<Link>>,
    image: Option<Image>,
}

impl Inline {
    pub(super) fn text(&self) -> String {
        self.spans.iter().map(|span| span.text.as_str()).collect()
    }
}

impl Open {
    fn span(&self, text: String, code: bool) -> Span {
        Span {
            text,
            style: Style {
                strong: self.strong > 0,
                emphasis: self.emphasis > 0,
                code,
            },
            link: self.links.last().cloned().flatten(),
        }
    }
}

pub(super) const fn is_inline(event: &Event<'_>) -> bool {
    match event {
        Event::Start(tag) => matches!(
            tag,
            Tag::Emphasis
                | Tag::Strong
                | Tag::Strikethrough
                | Tag::Superscript
                | Tag::Subscript
                | Tag::Link {
                    link_type: _,
                    dest_url: _,
                    title: _,
                    id: _
                }
                | Tag::Image {
                    link_type: _,
                    dest_url: _,
                    title: _,
                    id: _
                }
        ),
        Event::End(tag) => matches!(
            tag,
            TagEnd::Emphasis
                | TagEnd::Strong
                | TagEnd::Strikethrough
                | TagEnd::Superscript
                | TagEnd::Subscript
                | TagEnd::Link
                | TagEnd::Image
        ),
        Event::Text(_)
        | Event::Code(_)
        | Event::SoftBreak
        | Event::HardBreak
        | Event::InlineHtml(_)
        | Event::InlineMath(_)
        | Event::FootnoteReference(_)
        | Event::TaskListMarker(_) => true,
        Event::Html(_) | Event::DisplayMath(_) | Event::Rule => false,
    }
}

/// Takes the running text from `events` up to the first event that is
/// not part of it, which stays.
pub(super) fn take(events: &mut Peekable<Parser<'_>>) -> Inline {
    let mut inline = Inline::default();
    let mut open = Open::default();

    while let Some(event) = events.next_if(is_inline) {
        match event {
            Event::Start(tag) => start(tag, &mut open),
            Event::End(tag) => end(tag, &mut open, &mut inline),
            Event::Text(text) => {
                if let Some(image) = open.image.as_mut() {
                    image.alt.push_str(&text);
                } else {
                    inline.spans.push(open.span(text.into_string(), false));
                }
            }
            Event::Code(text) => inline.spans.push(open.span(text.into_string(), true)),
            Event::SoftBreak | Event::HardBreak => {
                if open.image.is_none() {
                    inline.spans.push(open.span(" ".into(), false));
                }
            }
            Event::InlineHtml(_)
            | Event::InlineMath(_)
            | Event::FootnoteReference(_)
            | Event::TaskListMarker(_)
            | Event::Html(_)
            | Event::DisplayMath(_)
            | Event::Rule => {}
        }
    }

    inline
}

fn start(tag: Tag<'_>, open: &mut Open) {
    if tag == Tag::Strong {
        open.strong += 1;
    } else if tag == Tag::Emphasis {
        open.emphasis += 1;
    } else if let Tag::Link {
        link_type: _,
        dest_url,
        title: _,
        id: _,
    } = tag
    {
        open.links.push(resolve(&dest_url));
    } else if let Tag::Image {
        link_type: _,
        dest_url,
        title: _,
        id: _,
    } = tag
    {
        open.image = Some(Image {
            src: dest_url.into_string(),
            alt: String::new(),
        });
    } else {
        // Strikethrough and the scripts keep the plain style.
    }
}

fn end(tag: TagEnd, open: &mut Open, inline: &mut Inline) {
    if tag == TagEnd::Strong {
        open.strong = open.strong.saturating_sub(1);
    } else if tag == TagEnd::Emphasis {
        open.emphasis = open.emphasis.saturating_sub(1);
    } else if tag == TagEnd::Link {
        open.links.pop();
    } else if tag == TagEnd::Image {
        inline.images.extend(open.image.take());
    } else {
        // Strikethrough and the scripts kept the plain style.
    }
}

/// Where a link's address goes, or `None` for anything but another page,
/// a heading or a web address.
fn resolve(address: &str) -> Option<Link> {
    if let Some(anchor) = address.strip_prefix('#') {
        return Some(Link::Anchor(anchor.to_owned()));
    }

    if address.starts_with("https://") || address.starts_with("http://") {
        return Some(Link::Web(address.to_owned()));
    }

    if address.contains(':') {
        return None;
    }

    let address = address.strip_prefix("./").unwrap_or(address);
    let (file, anchor) = match address.split_once('#') {
        Some((file, anchor)) => (file, Some(anchor.to_owned())),
        None => (address, None),
    };

    let page = std::path::Path::new(file)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"));

    page.then(|| Link::Page {
        file: file.to_owned(),
        anchor,
    })
}
