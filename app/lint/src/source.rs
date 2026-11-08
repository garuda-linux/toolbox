//! A parsed file: its comments without markers, where they stand, and the
//! code next to them.

mod code;
mod comment;
#[cfg(test)]
mod tests;

use std::path::Path;

pub use code::looks_like_code;
pub use comment::{Comment, Kind, Unit};
use tree_sitter::{Language, Node, Parser, Point, Tree};

/// A token of code: what the restating rules compare comment words with.
#[derive(Clone, Copy, Debug)]
pub struct Leaf<'t> {
    pub at: Point,
    pub text: &'t str,
    pub kind: &'t str,
    pub named: bool,
}

pub struct Source<'t> {
    pub text: &'t str,
    pub comments: Vec<Comment<'t>>,
    pub leaves: Vec<Leaf<'t>>,
    pub root: Node<'t>,
    pub lines: Vec<&'t str>,
}

impl<'t> Source<'t> {
    pub fn new(text: &'t str, tree: &'t Tree) -> Self {
        let mut source = Self {
            text,
            comments: Vec::new(),
            leaves: Vec::new(),
            root: tree.root_node(),
            lines: text.split('\n').collect(),
        };

        source.collect(tree.root_node());
        source
    }

    pub fn node_text(&self, node: Node<'_>) -> &'t str {
        node.utf8_text(self.text.as_bytes()).unwrap_or_default()
    }

    /// One logical comment each: runs of `//` lines and every other comment.
    pub fn units(&self) -> Vec<Unit<'_, 't>> {
        let mut units: Vec<Unit<'_, 't>> = Vec::new();

        for comment in self.comments.iter().filter(|comment| !comment.directive) {
            match units.last_mut() {
                Some(unit) if unit.last().continues_with(comment) => unit.comments.push(comment),
                _ => units.push(Unit {
                    comments: vec![comment],
                }),
            }
        }

        units
    }

    /// The tokens of the line a comment describes: the next line of code for a
    /// comment on its own lines, the same line for a trailing one.
    pub fn adjacent_code(&self, unit: &Unit<'_, 't>) -> Vec<Leaf<'t>> {
        let last = unit.last();

        if last.alone {
            let next_row = self
                .leaves
                .iter()
                .find(|leaf| leaf.at.row > last.end.row)
                .map(|leaf| leaf.at.row);

            return next_row.map_or_else(Vec::new, |row| self.leaves_on_row(row, usize::MAX));
        }

        let start = last.node.start_position();
        if unit.comments.len() == 1 && start.row == last.end.row {
            return self.leaves_on_row(start.row, start.column);
        }

        Vec::new()
    }

    /// Inside a function or closure body, where the code should speak for itself.
    pub fn in_function_body(node: Node<'_>) -> bool {
        ancestors(node).any(|ancestor| ancestor.kind() == "block")
    }

    /// The item a doc comment documents, past attributes and other comments.
    pub fn documented_item(unit: &Unit<'_, 't>) -> Option<Node<'t>> {
        let mut next = unit.last().node.next_named_sibling();

        while let Some(node) = next {
            if !is_comment(node.kind()) && node.kind() != "attribute_item" {
                return Some(node);
            }

            next = node.next_named_sibling();
        }

        None
    }

    /// Every node of `kind`, in source order.
    pub fn nodes_of_kind(&self, kind: &str) -> Vec<Node<'t>> {
        let mut found = Vec::new();
        let mut stack = vec![self.root];

        while let Some(node) = stack.pop() {
            if node.kind() == kind {
                found.push(node);
            }

            let mut cursor = node.walk();
            stack.extend(node.children(&mut cursor));
        }

        found.sort_by_key(Node::start_byte);
        found
    }

    fn leaves_on_row(&self, row: usize, before_column: usize) -> Vec<Leaf<'t>> {
        self.leaves
            .iter()
            .filter(|leaf| leaf.at.row == row && leaf.at.column < before_column)
            .copied()
            .collect()
    }

    fn line(&self, row: usize) -> &'t str {
        self.lines.get(row).copied().unwrap_or_default()
    }

    fn collect(&mut self, root: Node<'t>) {
        let mut stack = vec![root];

        while let Some(node) = stack.pop() {
            if is_comment(node.kind()) {
                let comment = Comment::new(self, node);
                self.comments.push(comment);
            } else if node.child_count() == 0 {
                self.leaves.push(Leaf {
                    at: node.start_position(),
                    text: self.node_text(node),
                    kind: node.kind(),
                    named: node.is_named(),
                });
            } else {
                let mut cursor = node.walk();
                stack.extend(node.children(&mut cursor));
            }
        }

        self.comments
            .sort_by_key(|comment| comment.node.start_byte());
        self.leaves
            .sort_by_key(|leaf| (leaf.at.row, leaf.at.column));
    }
}

/// The linter reads Rust only.
pub fn is_linted(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "rs")
}

pub fn is_comment(kind: &str) -> bool {
    kind == "line_comment" || kind == "block_comment"
}

pub fn parse(text: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    parser
        .set_language(&Language::new(tree_sitter_rust::LANGUAGE))
        .ok()?;
    parser.parse(text, None)
}

fn ancestors(node: Node<'_>) -> impl Iterator<Item = Node<'_>> {
    std::iter::successors(node.parent(), Node::parent)
}
