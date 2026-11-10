//! `domain-registered`: an `impl Domain for X` with no `.with::<X>()` in
//! `domains::all()`. A saved change of it is refused as an unknown domain
//! after a restart, and the registry's checks never see it.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::finding::Finding;
use crate::pattern::re;
use crate::source::{Place, Source};

pub(super) const RULE: &str = "domain-registered";

/// The file of `domains::all()`, from the repository root.
pub const REGISTRY: &str = "crates/core/src/domains.rs";

static WITH: LazyLock<Regex> = LazyLock::new(|| re(r"\.with::<(?:\w+::)*(\w+)>\(\)"));

/// The type names `domains::all()` registers, from its file's text.
pub fn registered(text: &str) -> BTreeSet<String> {
    WITH.captures_iter(text)
        .filter_map(|found| found.get(1))
        .map(|name| name.as_str().to_owned())
        .collect()
}

pub fn check(
    source: &Source<'_>,
    place: &Place<'_>,
    registered: Option<&BTreeSet<String>>,
    findings: &mut Vec<Finding>,
) {
    let Some(registered) = registered else {
        return;
    };

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
