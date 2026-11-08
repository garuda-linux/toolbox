//! Whether comment text is code: it parses, wrapped where statements are
//! allowed, and says more than a bare word or label, which prose never does.

use std::sync::LazyLock;

use regex::Regex;
use tree_sitter::Node;

use super::parse;
use crate::pattern::re;

static PROSE_MARKER: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?i)^(TODO|FIXME|HACK|NOTE|XXX|REVIEW)\b|^https?://"));

/// Single expressions that prose can parse as: one word, a path, a literal.
const TRIVIAL: &[&str] = &[
    "identifier",
    "field_expression",
    "scoped_identifier",
    "string_literal",
    "raw_string_literal",
    "char_literal",
    "integer_literal",
    "float_literal",
    "true",
    "false",
    "boolean_literal",
    "self",
    "empty_statement",
    "line_comment",
    "block_comment",
];

pub fn looks_like_code(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() || PROSE_MARKER.is_match(trimmed) {
        return false;
    }

    parses_as_code(trimmed, "", "") || parses_as_code(trimmed, "fn wrapper() {\n", "\n}")
}

fn parses_as_code(text: &str, before: &str, after: &str) -> bool {
    let wrapped = format!("{before}{text}{after}");
    let Some(tree) = parse(&wrapped) else {
        return false;
    };

    let root = tree.root_node();
    if root.has_error() {
        return false;
    }

    let body = if before.is_empty() {
        Some(root)
    } else {
        find_body(root)
    };

    body.is_some_and(|body| {
        let mut cursor = body.walk();
        let mut statements = body.named_children(&mut cursor).peekable();
        statements.peek().is_some() && !statements.all(is_trivial)
    })
}

fn find_body(root: Node<'_>) -> Option<Node<'_>> {
    let mut stack = vec![root];

    while let Some(node) = stack.pop() {
        if node.kind() == "block" {
            return Some(node);
        }

        let mut cursor = node.walk();
        stack.extend(node.named_children(&mut cursor));
    }

    None
}

/// An expression statement is trivial when its value is.
fn is_trivial(node: Node<'_>) -> bool {
    if TRIVIAL.contains(&node.kind()) {
        return true;
    }

    let mut cursor = node.walk();
    node.kind() == "expression_statement"
        && node
            .named_children(&mut cursor)
            .last()
            .is_some_and(is_trivial)
}
