//! `domain-registered`: an `impl Domain for X` with no `.with::<X>()` in
//! `domains::all()`. A saved change of it is refused as an unknown domain
//! after a restart, and the registry's checks never see it.

use std::collections::BTreeSet;

use crate::finding::Finding;
use crate::source::{Place, Source, parse};

pub(super) const RULE: &str = "domain-registered";

/// The file of `domains::all()`, from the repository root.
pub const REGISTRY: &str = "crates/core/src/domains.rs";

/// The type names `domains::all()` registers, from its file's text. Only the
/// top-level `fn all` counts, so a comment or a test module does not.
pub fn registered(text: &str) -> Result<BTreeSet<String>, String> {
    let tree = parse(text).ok_or_else(|| format!("cannot parse {REGISTRY}"))?;
    let source = Source::new(text, &tree);

    let mut cursor = source.root.walk();
    let body = source
        .root
        .named_children(&mut cursor)
        .find(|item| {
            item.kind() == "function_item"
                && item
                    .child_by_field_name("name")
                    .is_some_and(|name| source.node_text(name) == "all")
        })
        .and_then(|item| item.child_by_field_name("body"))
        .ok_or_else(|| format!("no top-level `fn all` in {REGISTRY}"))?;

    Ok(source
        .nodes_of_kind("call_expression")
        .into_iter()
        .filter(|call| body.start_byte() <= call.start_byte() && call.end_byte() <= body.end_byte())
        .filter_map(|call| with_type(&source, call))
        .collect())
}

/// `X` of a `.with::<X>()` call, `None` for any other call.
fn with_type<'t>(source: &Source<'t>, call: tree_sitter::Node<'t>) -> Option<String> {
    let function = call.child_by_field_name("function")?;
    if function.kind() != "generic_function" {
        return None;
    }

    let method = function.child_by_field_name("function")?;
    if method.kind() != "field_expression"
        || source.node_text(method.child_by_field_name("field")?) != "with"
    {
        return None;
    }

    let arguments = function.child_by_field_name("type_arguments")?;
    let mut cursor = arguments.walk();
    let argument = arguments.named_children(&mut cursor).next()?;
    let name = match argument.kind() {
        "type_identifier" => argument,
        "scoped_type_identifier" => argument.child_by_field_name("name")?,
        _ => return None,
    };

    Some(source.node_text(name).to_owned())
}

pub fn check(
    source: &Source<'_>,
    place: &Place<'_>,
    registered: &BTreeSet<String>,
    findings: &mut Vec<Finding>,
) {
    if place.is_test() {
        return;
    }

    for item in source.nodes_of_kind("impl_item") {
        let is_domain = item
            .child_by_field_name("trait")
            .map(|name| source.node_text(name))
            .is_some_and(|name| name == "Domain" || name.ends_with("::Domain"));

        let Some(kind) = item.child_by_field_name("type") else {
            continue;
        };

        let name = source.node_text(kind);
        if !is_domain || registered.contains(name) {
            continue;
        }

        findings.push(
            Finding::new(
                item.start_position(),
                RULE,
                format!("`{name}` implements `Domain`, but `domains::all()` lacks it."),
                "A saved change of it is refused as an unknown domain after a restart, and the registry's checks never see it.",
            )
            .fix(format!("Add `.with::<{name}>()?` to `domains::all()` in `{REGISTRY}`.")),
        );
    }
}
