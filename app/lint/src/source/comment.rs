//! Comment text without markers, and runs of `//` lines read as one comment.

use std::sync::LazyLock;

use regex::Regex;
use tree_sitter::{Node, Point};

use super::Source;
use crate::pattern::re;

/// Tool directives are instructions, not prose.
static DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| re(r"^\s*toolbox-lint:"));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Line,
    Block,
    /// `///` and `/** */`.
    Doc,
    /// `//!` and `/*! */`, documentation of the enclosing module.
    InnerDoc,
}

#[derive(Clone, Debug)]
pub struct TextLine {
    pub at: Point,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct Comment<'t> {
    pub node: Node<'t>,
    pub kind: Kind,
    pub lines: Vec<TextLine>,
    /// Where the comment text ends (a line comment node includes the newline).
    pub end: Point,
    /// Nothing but whitespace shares its first and last line.
    pub alone: bool,
    pub directive: bool,
    pub slashes: bool,
}

/// Comments read as one: consecutive `//` lines, or a single other comment.
pub struct Unit<'a, 't> {
    pub comments: Vec<&'a Comment<'t>>,
}

impl<'t> Comment<'t> {
    pub fn new(source: &Source<'t>, node: Node<'t>) -> Self {
        let raw = source.node_text(node).trim_end_matches(['\n', '\r']);
        let start = node.start_position();
        let slashes = raw.starts_with("//");
        let (kind, lines) = if slashes {
            line_comment(raw, start)
        } else {
            block_comment(raw, start)
        };

        let last_row = start.row + raw.matches('\n').count();
        let last_line = source.line(last_row);
        let end_column = if last_row == start.row {
            start.column + raw.len()
        } else {
            raw.rsplit('\n').next().map_or(0, str::len)
        };

        let end = Point::new(last_row, end_column);

        let before = source
            .line(start.row)
            .get(..start.column)
            .unwrap_or_default();

        let after = last_line.get(end_column..).unwrap_or_default();
        let directive = lines
            .first()
            .is_some_and(|line| DIRECTIVE.is_match(&line.text));

        Self {
            node,
            kind,
            lines,
            end,
            alone: before.trim().is_empty() && after.trim().is_empty(),
            directive,
            slashes,
        }
    }

    pub fn text(&self) -> String {
        join_lines(self.lines.iter())
    }

    pub fn start(&self) -> Point {
        self.node.start_position()
    }

    /// `next` belongs to the same run of `//` lines.
    pub fn continues_with(&self, next: &Comment<'_>) -> bool {
        self.slashes
            && next.slashes
            && self.alone
            && next.alone
            && self.kind == next.kind
            && next.start().row == self.end.row + 1
            && next.start().column == self.start().column
    }
}

impl<'a, 't> Unit<'a, 't> {
    pub fn first(&self) -> &'a Comment<'t> {
        self.comments
            .first()
            .copied()
            .unwrap_or_else(|| unreachable_unit())
    }

    pub fn last(&self) -> &'a Comment<'t> {
        self.comments
            .last()
            .copied()
            .unwrap_or_else(|| unreachable_unit())
    }

    pub fn kind(&self) -> Kind {
        self.first().kind
    }

    pub fn start(&self) -> Point {
        self.first().start()
    }

    pub fn lines(&self) -> impl Iterator<Item = &'a TextLine> {
        self.comments
            .iter()
            .flat_map(|comment| comment.lines.iter())
    }

    pub fn text(&self) -> String {
        join_lines(self.lines())
    }

    /// The lines before the first section heading or code example.
    pub fn description(&self) -> impl Iterator<Item = &'a TextLine> {
        self.lines().take_while(|line| {
            let text = line.text.trim_start();
            !(text.starts_with("# ") || text.starts_with("```"))
        })
    }

    pub fn height(&self) -> usize {
        self.last().end.row - self.start().row + 1
    }
}

/// Units are built with at least one comment and only grow.
#[expect(clippy::panic, reason = "a unit is never empty")]
fn unreachable_unit() -> ! {
    panic!("comment unit without comments")
}

fn line_comment(raw: &str, start: Point) -> (Kind, Vec<TextLine>) {
    let rust_doc = !raw.starts_with("////");
    let (kind, marker) = if rust_doc && raw.starts_with("///") {
        (Kind::Doc, 3)
    } else if rust_doc && raw.starts_with("//!") {
        (Kind::InnerDoc, 3)
    } else {
        (Kind::Line, 2)
    };

    let rest = raw.get(marker..).unwrap_or_default();
    let (skipped, text) = rest
        .strip_prefix(' ')
        .map_or((marker, rest), |text| (marker + 1, text));

    let line = TextLine {
        at: Point::new(start.row, start.column + skipped),
        text: text.trim_end().to_owned(),
    };

    (kind, vec![line])
}

fn block_comment(raw: &str, start: Point) -> (Kind, Vec<TextLine>) {
    let doc = raw.starts_with("/**") && !raw.starts_with("/**/");
    let (kind, marker) = if doc {
        (Kind::Doc, 3)
    } else if raw.starts_with("/*!") {
        (Kind::InnerDoc, 3)
    } else {
        (Kind::Block, 2)
    };

    let body = raw.get(marker..).unwrap_or_default();
    let body = body.strip_suffix("*/").unwrap_or(body);
    let mut lines: Vec<TextLine> = body
        .split('\n')
        .enumerate()
        .map(|(index, line)| {
            if index == 0 {
                let text = line.strip_prefix(' ').unwrap_or(line);
                let column = start.column + marker + line.len() - text.len();
                return TextLine {
                    at: Point::new(start.row, column),
                    text: text.trim_end().to_owned(),
                };
            }

            let gutter = line.trim_start();
            let gutter = gutter.strip_prefix('*').unwrap_or(gutter);
            let text = gutter.strip_prefix(' ').unwrap_or(gutter);
            TextLine {
                at: Point::new(start.row + index, line.len() - text.len()),
                text: text.trim_end().to_owned(),
            }
        })
        .collect();

    while lines.first().is_some_and(|line| line.text.is_empty()) {
        lines.remove(0);
    }

    while lines.last().is_some_and(|line| line.text.is_empty()) {
        lines.pop();
    }

    (kind, lines)
}

fn join_lines<'a>(lines: impl Iterator<Item = &'a TextLine>) -> String {
    lines
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}
