//! String literals, and the programs `Command::new` calls name with one. A
//! program in a variable or a `Program` is out of reach of a syntax rule.

use tree_sitter::{Node, Point};

use super::Source;

const LITERALS: [&str; 2] = ["string_literal", "raw_string_literal"];

/// `std::process::Command`, tokio's and `PtyCommand` all end in this.
const SPAWN: &str = "Command::new";

pub struct Literal<'t> {
    pub at: Point,
    pub value: &'t str,
}

/// Every string literal of the file, in source order.
pub fn string_literals<'t>(source: &Source<'t>) -> Vec<Literal<'t>> {
    let mut found: Vec<Literal<'t>> = LITERALS
        .iter()
        .flat_map(|kind| source.nodes_of_kind(kind))
        .map(|node| literal(source, node))
        .collect();

    found.sort_by_key(|literal| (literal.at.row, literal.at.column));
    found
}

/// The first argument of every `Command::new` call that is a string literal.
pub fn spawned_literals<'t>(source: &Source<'t>) -> Vec<Literal<'t>> {
    source
        .nodes_of_kind("call_expression")
        .into_iter()
        .filter_map(|call| {
            let function = call.child_by_field_name("function")?;
            if !source.node_text(function).ends_with(SPAWN) {
                return None;
            }

            let first = call.child_by_field_name("arguments")?.named_child(0)?;
            LITERALS
                .contains(&first.kind())
                .then(|| literal(source, first))
        })
        .collect()
}

/// The text between the quotes, past an `r` and its hashes.
fn literal<'t>(source: &Source<'t>, node: Node<'_>) -> Literal<'t> {
    let value = source
        .node_text(node)
        .trim_start_matches('r')
        .trim_matches('#')
        .trim_matches('"');

    Literal {
        at: node.start_position(),
        value,
    }
}
