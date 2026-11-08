//! `no-restating-doc`: doc comments that only repeat the signature,
//! such as "Gets the invoice lines." on `get_invoice_lines`.

use std::collections::HashSet;

use crate::finding::Finding;
use crate::source::{Kind, Source, Unit};
use crate::words::{WHY_WORDS, identifier_words, prose_words};

pub(super) const RULE: &str = "no-restating-doc";

const GENERIC: &[&str] = &[
    "method",
    "function",
    "class",
    "service",
    "component",
    "helper",
    "value",
    "instance",
    "object",
    "property",
    "field",
    "variable",
    "constant",
    "handler",
    "data",
    "info",
    "information",
    "list",
    "parameter",
    "argument",
    "struct",
    "enum",
    "variant",
    "type",
    "output",
    "path",
    "file",
    "directory",
    "folder",
    "absolute",
    "relative",
    "option",
    "setting",
    "priority",
    "command",
    "package",
    "input",
    "revision",
    "name",
    "condition",
    "host",
    "entry",
    "user",
    "text",
    "number",
];

pub fn check(source: &Source<'_>, findings: &mut Vec<Finding>) {
    for unit in source
        .units()
        .iter()
        .filter(|unit| unit.kind() == Kind::Doc)
    {
        let Some(name) = Source::documented_item(unit)
            .and_then(|item| item.child_by_field_name("name"))
            .map(|name| source.node_text(name))
        else {
            continue;
        };

        let name_words: HashSet<String> = identifier_words(name).into_iter().collect();

        if only_known_words(&description(unit), &name_words) {
            findings.push(
                Finding::new(
                    unit.start(),
                    RULE,
                    format!("The doc description of `{name}` only repeats its name."),
                    "A doc comment that restates the signature costs reading time and adds nothing the editor does not already show.",
                )
                .fix("Delete it, or say what the name cannot: preconditions, side effects, units, the rule behind it."),
            );
        }
    }
}

fn description(unit: &Unit<'_, '_>) -> String {
    unit.description()
        .map(|line| line.text.trim())
        .collect::<Vec<_>>()
        .join(" ")
}

fn only_known_words(text: &str, known: &HashSet<String>) -> bool {
    if text.trim().is_empty() || WHY_WORDS.is_match(text) {
        return false;
    }

    let words = prose_words(text);
    !words.is_empty()
        && words
            .iter()
            .all(|word| known.contains(word) || GENERIC.contains(&word.as_str()))
}
