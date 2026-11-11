use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::features::diagnostics::{Known, redact};
use toolbox_services::diagnostics::page::Page;

use super::{page, report};

fn naming(user: &str, plain: &str) -> Page {
    let known = Known {
        user: Some(user.to_owned()),
        ..Known::default()
    };

    Page {
        report: plain.to_owned(),
        shareable: Some(redact(plain, &known)),
        ..report()
    }
}

#[test]
fn the_question_shows_the_redacted_text_and_how_much_was_replaced() {
    let mut harness = page(naming("ada", "login of ada from 8.8.4.4\n"));

    harness.get_by_label("Share report…").click();
    harness.run();

    let preview = harness
        .get_by_role_and_label(Role::MultilineTextInput, "The text that will be uploaded")
        .value()
        .unwrap_or_default();

    assert_eq!(preview, "login of <user> from <ip-1>\n");
    assert!(
        harness
            .query_by_label_contains("2 values that name you or this computer were replaced")
            .is_some()
    );
}

#[test]
fn share_waits_for_the_redacted_text() {
    let harness = page(Page {
        shareable: None,
        ..report()
    });

    let share = harness.get_by_label("Share report…");

    assert!(
        share.accesskit_node().is_disabled(),
        "the redacted report is not there yet"
    );
}

#[test]
fn the_search_counts_its_matches_in_the_text_to_upload() {
    let mut harness = page(naming("ada", "kernel: oops\nkernel: oops again\n"));

    harness.get_by_label("Share report…").click();
    harness.run();
    harness.get_by_label("Search the text").click();
    harness.run();
    harness.get_by_label("Search the text").type_text("oops");
    harness.run();

    assert!(harness.query_by_label("1 of 2").is_some());

    harness.get_by_label("Next match").click();
    harness.run();

    assert!(harness.query_by_label("2 of 2").is_some());
}
