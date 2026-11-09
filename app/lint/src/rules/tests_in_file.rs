//! `tests-in-file`: an inline `mod tests { … }` under `src/`. Tests live in
//! `foo/tests.rs`, so the 400 lines of a file are its code alone.

use crate::finding::Finding;
use crate::source::{Place, Source};

pub(super) const RULE: &str = "tests-in-file";

pub fn check(source: &Source<'_>, place: &Place<'_>, findings: &mut Vec<Finding>) {
    if !place.in_src() {
        return;
    }

    for module in source.nodes_of_kind("mod_item") {
        let named_tests = module
            .child_by_field_name("name")
            .is_some_and(|name| source.node_text(name) == "tests");

        if !named_tests || module.child_by_field_name("body").is_none() {
            continue;
        }

        findings.push(
            Finding::new(
                module.start_position(),
                RULE,
                "An inline `mod tests { … }`.".to_owned(),
                "Inline tests count against the file's 400 lines and hide among the code.",
            )
            .fix("Move the body to `tests.rs` beside the file, in the folder named after it, and keep `#[cfg(test)] mod tests;`."),
        );
    }
}
