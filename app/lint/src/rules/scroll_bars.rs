//! `scroll-bars`: a `ScrollArea` shown with its own `show`. egui draws its
//! bars invisible, so only the helpers in `toolbox_gui_kit::theme` paint
//! Darkly's handle.

use crate::finding::Finding;
use crate::source::{Place, Source};

pub(super) const RULE: &str = "scroll-bars";

/// The one file that calls `ScrollArea::show` itself.
const HELPERS: &str = "crates/gui-kit/src/theme/scrollbar.rs";

const SHOWS: [&str; 3] = ["show", "show_viewport", "show_rows"];

pub fn check(source: &Source<'_>, place: &Place<'_>, findings: &mut Vec<Finding>) {
    if place.is_test() || place.is(HELPERS) {
        return;
    }

    for call in source.nodes_of_kind("call_expression") {
        let Some(function) = call.child_by_field_name("function") else {
            continue;
        };

        if function.kind() != "field_expression" {
            continue;
        }

        let Some(field) = function.child_by_field_name("field") else {
            continue;
        };

        let Some(receiver) = function.child_by_field_name("value") else {
            continue;
        };

        let name = source.node_text(field);
        if !SHOWS.contains(&name) || !source.node_text(receiver).contains("ScrollArea") {
            continue;
        }

        findings.push(
            Finding::new(
                field.start_position(),
                RULE,
                format!("`.{name}(` shows a `ScrollArea` without its bars."),
                "egui draws the bars of a `ScrollArea` invisible, so the page shows no scroll bar.",
            )
            .fix("Show it with `toolbox_gui_kit::theme::show_scroll`, `show_scroll_rows` or `show_scroll_hidden`, or allow this line with the reason."),
        );
    }
}
