//! A guide page as terminal text: headings in bold, prose wrapped to the
//! width, lists and tables lined up, notes labelled and code indented.

use std::fmt::Write as _;

use toolbox_cli::output::Paint;
use toolbox_guide::{Block, Callout, CalloutKind, Link, List, Page, Span, Table};

const CODE_INDENT: &str = "    ";
const INDENT: &str = "  ";
/// The widest the first column of a table gets before its text wraps.
const FIRST_COLUMN: usize = 32;
/// The narrowest text gets, so a tiny width still shows words.
const MIN_WIDTH: usize = 20;

/// `page` as text at most `width` columns wide, code lines excepted.
pub fn page(page: &Page, paint: Paint, width: usize) -> String {
    let mut out = String::new();
    blocks(&mut out, &page.blocks, paint, width, page.part.as_deref());
    format!("{}\n", out.trim_end())
}

fn blocks(out: &mut String, blocks: &[Block], paint: Paint, width: usize, part: Option<&str>) {
    for block in blocks {
        one(out, block, paint, width.max(MIN_WIDTH), part);
    }
}

fn one(out: &mut String, block: &Block, paint: Paint, width: usize, part: Option<&str>) {
    match block {
        Block::Heading(heading) => {
            let summary = if heading.level == 1 { part } else { None };
            writeln!(out, "{}\n", paint.title(&words(&heading.spans), summary)).ok();
        }
        Block::Paragraph(spans) => {
            for line in wrap(&words(spans), width) {
                writeln!(out, "{line}").ok();
            }

            out.push('\n');
        }
        Block::List(list) => items(out, list, paint, width),
        Block::Table(table) => rows(out, table, paint, width),
        Block::Code(code) => {
            for line in code.text.lines() {
                writeln!(out, "{CODE_INDENT}{}", paint.dim(line)).ok();
            }

            out.push('\n');
        }
        Block::Callout(callout) => note(out, callout, paint, width),
        Block::Image(image) => {
            writeln!(out, "{}\n", paint.dim(&format!("[{}]", image.alt))).ok();
        }
        Block::Rule => {
            writeln!(out, "{}\n", paint.dim(&"-".repeat(width.min(40)))).ok();
        }
    }
}

/// Each item behind its bullet or number, its own lines lined up under
/// its first.
fn items(out: &mut String, list: &List, paint: Paint, width: usize) {
    for (at, item) in list.items.iter().enumerate() {
        let marker = list.start.map_or_else(
            || "•".to_owned(),
            |first| {
                let number = u64::try_from(at).map_or(first, |at| first.saturating_add(at));
                format!("{number}.")
            },
        );

        let lead = format!("{INDENT}{marker} ");
        let hang = " ".repeat(lead.chars().count());
        let mut inner = String::new();
        blocks(
            &mut inner,
            item,
            paint,
            width.saturating_sub(hang.len()),
            None,
        );

        for (row, line) in inner.trim_end().lines().enumerate() {
            let front = if row == 0 {
                lead.as_str()
            } else {
                hang.as_str()
            };

            if line.is_empty() {
                out.push('\n');
            } else {
                writeln!(out, "{front}{line}").ok();
            }
        }
    }

    out.push('\n');
}

fn rows(out: &mut String, table: &Table, paint: Paint, width: usize) {
    let cell = |cells: &[Vec<Span>], at: usize| {
        cells.get(at).map_or_else(String::new, |spans| words(spans))
    };

    let first = table
        .rows
        .iter()
        .chain(std::iter::once(&table.head))
        .map(|row| cell(row, 0).chars().count())
        .max()
        .unwrap_or(0)
        .min(FIRST_COLUMN);

    let rest = width
        .saturating_sub(INDENT.len() + first + 2)
        .max(MIN_WIDTH);

    let mut row = |cells: &[Vec<Span>], bold: bool| {
        let left = wrap(&cell(cells, 0), first);
        let others: Vec<String> = (1..cells.len()).map(|at| cell(cells, at)).collect();
        let right = wrap(&others.join("  "), rest);
        for at in 0..left.len().max(right.len()) {
            let one = left.get(at).map_or("", String::as_str);
            let two = right.get(at).map_or("", String::as_str);
            let pad = first.saturating_sub(one.chars().count());
            let line = format!("{one}{}  {two}", " ".repeat(pad));
            let line = line.trim_end();
            let shown = if bold {
                paint.title(line, None)
            } else {
                line.to_owned()
            };

            writeln!(out, "{INDENT}{shown}").ok();
        }
    };

    row(&table.head, true);
    for cells in &table.rows {
        row(cells, false);
    }

    out.push('\n');
}

fn note(out: &mut String, callout: &Callout, paint: Paint, width: usize) {
    let label = match callout.kind {
        CalloutKind::Note => Some("Note:"),
        CalloutKind::Warning => Some("Warning:"),
        CalloutKind::Quote => None,
    };

    let mut inner = String::new();
    for (at, block) in callout.blocks.iter().enumerate() {
        if let (0, Some(label), Block::Paragraph(spans)) = (at, label, block) {
            let text = format!("{label} {}", words(spans));
            for (row, line) in wrap(&text, width.saturating_sub(INDENT.len()))
                .iter()
                .enumerate()
            {
                let line = if row == 0 {
                    line.replacen(label, &paint.question(label), 1)
                } else {
                    line.clone()
                };

                writeln!(inner, "{line}").ok();
            }

            inner.push('\n');
        } else {
            one(
                &mut inner,
                block,
                paint,
                width.saturating_sub(INDENT.len()),
                None,
            );
        }
    }

    for line in inner.trim_end().lines() {
        if line.is_empty() {
            out.push('\n');
        } else {
            writeln!(out, "{INDENT}{line}").ok();
        }
    }

    out.push('\n');
}

fn words(spans: &[Span]) -> String {
    let mut text = String::new();
    for span in spans {
        text.push_str(&span.text);
        if let Some(Link::Web(address)) = &span.link
            && *address != span.text
        {
            write!(text, " <{address}>").ok();
        }
    }

    text
}

/// `text` in lines of at most `width` characters, broken between words.
/// A word longer than a line gets a line of its own.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let wanted = line.chars().count() + 1 + word.chars().count();
        if !line.is_empty() && wanted > width {
            lines.push(std::mem::take(&mut line));
        }

        if !line.is_empty() {
            line.push(' ');
        }

        line.push_str(word);
    }

    if !line.is_empty() {
        lines.push(line);
    }

    lines
}
