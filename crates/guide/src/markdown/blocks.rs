use std::iter::Peekable;

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use super::anchor::Anchors;
use super::inline::{self, Inline};
use super::{Block, Callout, CalloutKind, Cell, Code, Heading, List, Table};

type Events<'text> = Peekable<Parser<'text>>;

pub(super) fn parse(markdown: &str) -> Vec<Block> {
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    let mut events = Parser::new_ext(markdown, options).peekable();
    let mut anchors = Anchors::default();
    blocks(&mut events, &mut anchors)
}

fn blocks(events: &mut Events<'_>, anchors: &mut Anchors) -> Vec<Block> {
    let mut out = Vec::new();
    while let Some(next) = events.peek() {
        if matches!(next, Event::End(_)) {
            break;
        }

        // A tight list item holds its text without a paragraph.
        if inline::is_inline(next) {
            paragraph(inline::take(events), &mut out);
        } else if let Some(event) = events.next() {
            block(event, events, anchors, &mut out);
        } else {
            break;
        }
    }

    out
}

fn block(event: Event<'_>, events: &mut Events<'_>, anchors: &mut Anchors, out: &mut Vec<Block>) {
    let Event::Start(tag) = event else {
        if matches!(event, Event::Rule) {
            out.push(Block::Rule);
        }

        return;
    };

    if tag == Tag::Paragraph {
        paragraph(inline::take(events), out);
        skip_to_end(events);
    } else if let Tag::Heading {
        level,
        id: _,
        classes: _,
        attrs: _,
    } = tag
    {
        let text = inline::take(events);
        skip_to_end(events);
        out.push(Block::Heading(Heading {
            level: number(level),
            anchor: anchors.unique(&text.text()),
            spans: text.spans,
        }));
    } else if let Tag::List(start) = tag {
        out.push(list(start, events, anchors));
    } else if let Tag::BlockQuote(_) = tag {
        let inner = blocks(events, anchors);
        skip_to_end(events);
        out.push(callout(inner));
    } else if let Tag::CodeBlock(kind) = tag {
        out.push(code(kind, events));
    } else if let Tag::Table(_) = tag {
        out.push(table(events));
    } else {
        skip_to_end(events);
    }
}

fn paragraph(text: Inline, out: &mut Vec<Block>) {
    let words = text.spans.iter().any(|span| !span.text.trim().is_empty());
    if words {
        out.push(Block::Paragraph(text.spans));
    }

    out.extend(text.images.into_iter().map(Block::Image));
}

fn list(start: Option<u64>, events: &mut Events<'_>, anchors: &mut Anchors) -> Block {
    let mut items = Vec::new();
    while let Some(event) = events.next() {
        if matches!(event, Event::Start(Tag::Item)) {
            items.push(blocks(events, anchors));
            skip_to_end(events);
        } else if matches!(event, Event::End(TagEnd::List(_))) {
            break;
        } else {
            // Nothing but items sits directly in a list.
        }
    }

    Block::List(List { start, items })
}

fn callout(mut blocks: Vec<Block>) -> Block {
    let mut kind = CalloutKind::Quote;
    if let Some(Block::Paragraph(spans)) = blocks.first_mut()
        && let Some(first) = spans.first()
        && first.style.strong
    {
        kind = match first.text.trim() {
            "Note:" => CalloutKind::Note,
            "Warning:" => CalloutKind::Warning,
            _ => CalloutKind::Quote,
        };
        if kind != CalloutKind::Quote {
            spans.remove(0);
            if let Some(next) = spans.first_mut() {
                next.text = next.text.trim_start().to_owned();
                if next.text.is_empty() {
                    spans.remove(0);
                }
            }
        }
    }

    Block::Callout(Callout { kind, blocks })
}

fn code(kind: CodeBlockKind<'_>, events: &mut Events<'_>) -> Block {
    let language = match kind {
        CodeBlockKind::Fenced(info) => info.split_whitespace().next().map(str::to_owned),
        CodeBlockKind::Indented => None,
    };

    let mut text = String::new();
    for event in events.by_ref() {
        if let Event::Text(part) = event {
            text.push_str(&part);
        } else if matches!(event, Event::End(_)) {
            break;
        } else {
            // A code block holds text alone.
        }
    }

    text.truncate(text.trim_end_matches('\n').len());
    Block::Code(Code { language, text })
}

fn table(events: &mut Events<'_>) -> Block {
    let mut head = Vec::new();
    let mut rows = Vec::new();
    while let Some(event) = events.next() {
        if matches!(event, Event::Start(Tag::TableHead)) {
            head = cells(events);
        } else if matches!(event, Event::Start(Tag::TableRow)) {
            rows.push(cells(events));
        } else if matches!(event, Event::End(TagEnd::Table)) {
            break;
        } else {
            // A table holds its head and rows alone.
        }
    }

    Block::Table(Table { head, rows })
}

fn cells(events: &mut Events<'_>) -> Vec<Cell> {
    let mut cells = Vec::new();
    while let Some(event) = events.next() {
        if matches!(event, Event::Start(Tag::TableCell)) {
            cells.push(inline::take(events).spans);
            skip_to_end(events);
        } else if matches!(event, Event::End(_)) {
            break;
        } else {
            // A head or row holds cells alone.
        }
    }

    cells
}

fn skip_to_end(events: &mut Events<'_>) {
    let mut depth = 0_usize;
    for event in events.by_ref() {
        if matches!(event, Event::Start(_)) {
            depth += 1;
        } else if matches!(event, Event::End(_)) {
            if depth == 0 {
                break;
            }

            depth -= 1;
        } else {
            // Text inside is dropped with its container.
        }
    }
}

const fn number(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}
