mod heights;
mod redacted;
mod scroll;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::features::diagnostics::{
    Collector, Known, Refusal, Selection, UploadFailure, redact,
};
use toolbox_gui_kit::widgets::ButtonKind;
use toolbox_privatebin::Paste;
use toolbox_services::diagnostics::page::{Page, State};
use toolbox_services::share::{Upload, UploadProblem};

use super::DiagnosticsPage;
use crate::command::{AppCommand, Command, DiagnosticsCommand};
use crate::diagnostics::Diagnostics;
use crate::page_service::Services;
use crate::testing::{PageFixture, PageState, button_kind};

type Shown = Harness<'static, PageState<DiagnosticsPage>>;

fn page(shown: Page) -> Shown {
    let mut harness = PageFixture {
        services: Services {
            diagnostics: Diagnostics::showing(shown),
            ..Services::default()
        },
        ..PageFixture::default()
    }
    .harness(DiagnosticsPage::default());

    harness.run();
    harness
}

fn sent(harness: &Shown) -> Vec<Command> {
    harness.state().fixture.sent()
}

fn report() -> Page {
    let plain = "kernel: hello\nkernel: oops\n";

    Page {
        state: State::Report,
        selection: "dmesg".to_owned(),
        report: plain.to_owned(),
        shareable: Some(redact(plain, &Known::default())),
        raw: "kernel: hello\n\u{1b}[31mkernel: oops\u{1b}[0m\n".to_owned(),
        ..Page::default()
    }
}

fn uploaded() -> Page {
    Page {
        share: Upload::Done(Paste {
            link: "https://bin.garudalinux.org/?abc#key".to_owned(),
            delete_link: "https://bin.garudalinux.org/?pasteid=abc&deletetoken=tok".to_owned(),
        }),
        ..report()
    }
}

#[test]
fn the_first_view_asks_for_a_report_and_collects_the_picked_one() {
    let mut harness = page(Page::default());

    assert!(
        harness
            .query_by_label("Choose a report and click Collect.")
            .is_some()
    );

    harness
        .get_by_role_and_label(Role::ComboBox, "Report to collect")
        .click();
    harness.run();
    harness.get_by_label("Kernel log").click();
    harness.run();
    harness.get_by_label("Collect").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(DiagnosticsCommand::Collect(Selection::One(
            Collector::Dmesg
        )))]
    );
}

#[test]
fn a_running_full_report_names_its_part_and_can_be_cancelled() {
    let mut harness = page(Page {
        state: State::Running,
        selection: "all".to_owned(),
        part: "journalctl".to_owned(),
        step: 3,
        ..Page::default()
    });

    assert!(
        harness
            .query_by_label_contains("Collecting Journal (3 of 6)")
            .is_some()
    );

    harness.get_by_label("Cancel").click();
    harness.run();

    assert_eq!(sent(&harness), [Command::from(DiagnosticsCommand::Cancel)]);
}

#[test]
fn a_report_shows_its_lines_and_is_shared_after_the_question() {
    let mut harness = page(report());

    assert!(harness.query_by_label("kernel: oops").is_some());

    harness.get_by_label("Share report…").click();
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Upload this report?")
            .is_some()
    );
    assert!(
        harness
            .query_by_label_contains("anyone who has the link can")
            .is_some()
    );

    harness.get_by_label("Upload").click();
    harness.run();

    assert_eq!(sent(&harness), [Command::from(DiagnosticsCommand::Upload)]);
}

#[test]
fn copying_the_report_says_so() {
    let mut harness = page(report());

    harness.get_by_label("Copy report").click();
    harness.run();

    assert!(matches!(
        sent(&harness).as_slice(),
        [Command::App(AppCommand::Toast { text, kind: _ })] if text == "Copied the report to the clipboard."
    ));
}

#[test]
fn a_refused_collection_says_why() {
    let harness = page(Page {
        state: State::Refused(Refusal::Dismissed),
        selection: "dmesg".to_owned(),
        ..Page::default()
    });

    assert!(
        harness
            .query_by_label_contains(
                "Kernel log was not collected: the password dialog was closed."
            )
            .is_some()
    );
}

#[test]
fn an_upload_shows_both_links_and_deleting_asks_first() {
    let mut harness = page(uploaded());

    assert!(
        harness
            .query_by_label("Link to the uploaded report")
            .is_some()
    );

    // Here and not in the app's snapshots: a real upload's key differs
    // each time.
    harness.snapshot("diagnostics/uploaded_dark");

    harness.get_by_label("Delete now…").click();
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Delete the report from the server?")
            .is_some()
    );

    harness.get_by_label("Cancel").click();
    harness.run();

    assert!(harness.query_by_role(Role::Dialog).is_none());
    assert_eq!(sent(&harness), []);
}

#[test]
fn a_failed_upload_names_the_cause_and_is_dismissed() {
    let mut harness = page(Page {
        share: Upload::Failed(UploadProblem {
            kind: UploadFailure::Network,
            detail: "dns error".to_owned(),
        }),
        ..report()
    });

    assert!(
        harness
            .query_by_label_contains("bin.garudalinux.org could not be reached")
            .is_some()
    );
    assert!(harness.query_by_label_contains("dns error").is_some());

    harness.get_by_label("Dismiss").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(DiagnosticsCommand::HideShare)]
    );
}

#[test]
fn the_description_starts_on_the_cards_content_edge_and_keeps_its_distance() {
    let harness = page(Page::default());
    let picker = harness.get_by_label("Report to collect").rect();
    let description = harness
        .get_by_label("Hardware, drivers and system overview from garuda-inxi.")
        .rect();

    let gap = toolbox_gui_kit::tokens::current(&harness.ctx)
        .sizes
        .control_gap;

    assert!(
        (description.left() - picker.left()).abs() < 1.0,
        "description starts at {} but the card's content at {}",
        description.left(),
        picker.left(),
    );
    assert!(
        description.top() - picker.bottom() >= gap,
        "only {} px between the picker and its description",
        description.top() - picker.bottom(),
    );
}

#[test]
fn collect_leads_until_an_upload_then_its_link_does() {
    let harness = page(report());
    assert_eq!(button_kind(&harness, "Collect"), ButtonKind::Primary);

    let shared = page(uploaded());

    assert_eq!(button_kind(&shared, "Collect"), ButtonKind::Soft);
    assert_eq!(button_kind(&shared, "Copy link"), ButtonKind::Primary);
}

#[test]
fn the_link_fields_start_where_their_words_start_and_pad_their_text() {
    let harness = page(uploaded());
    let words = harness
        .get_by_label_contains("Anyone who has this link")
        .rect()
        .left();

    let field = harness
        .get_by_role_and_label(Role::TextInput, "Link to the uploaded report")
        .rect()
        .left();

    let link =
        toolbox_gui_kit::testing::painted_text(&harness, "https://bin.garudalinux.org/?abc#key")
            .expect("the link is painted")
            .pos
            .x;

    let padding = harness.ctx.global_style().spacing.button_padding.x;
    assert!(
        (field - words).abs() < 1.0,
        "the field starts at {field}, its words at {words}"
    );
    assert!(
        (link - field - padding).abs() < 1.0,
        "the link starts {} px inside its field",
        link - field
    );
}

#[test]
fn each_link_has_its_label_above_and_its_buttons_below_a_gap_apart() {
    let harness = page(uploaded());
    let sizes = toolbox_gui_kit::tokens::current(&harness.ctx).sizes;
    let uploaded = "Uploaded. Anyone who has this link can read the report:";
    let delete = "Delete link: opening it removes the report from the server.";
    let parts = [
        (uploaded, "Link to the uploaded report", "Copy link"),
        (delete, delete, "Copy delete link"),
    ];

    for (label, field, button) in parts {
        let above = harness.get_by_role_and_label(Role::Label, label).rect();
        let field = harness.get_by_role_and_label(Role::TextInput, field).rect();
        let button = harness.get_by_label(button).rect();
        assert!(
            field.top() - above.bottom() >= sizes.caption_gap - 0.5,
            "{label}: {} px above its field",
            field.top() - above.bottom()
        );
        assert!(
            button.top() - field.bottom() >= sizes.control_gap - 0.5,
            "{label}: {} px between the field and its buttons",
            button.top() - field.bottom()
        );
    }

    let look = |text: &str| {
        let shape = toolbox_gui_kit::testing::painted_text(&harness, text).expect("painted");
        let format = &shape.galley.job.sections[0].format;
        (format.font_id.clone(), format.color)
    };

    assert_eq!(
        look(uploaded),
        look(delete),
        "two labels of one role differ"
    );
}

#[test]
fn the_text_to_upload_sits_in_a_frame_of_its_own() {
    let mut harness = page(report());
    harness.get_by_label("Share report…").click();
    harness.run();

    let preview = harness
        .get_by_role_and_label(Role::MultilineTextInput, "The text that will be uploaded")
        .rect();

    let cancel = harness.get_by_label("Cancel").rect();
    let inside = egui::pos2(preview.right() - 4.0, preview.bottom() - 4.0);
    let around = egui::pos2(preview.left() + 4.0, cancel.center().y);

    assert_ne!(
        toolbox_gui_kit::testing::fill_under(&harness, inside),
        toolbox_gui_kit::testing::fill_under(&harness, around),
        "the preview has no fill of its own"
    );
}

#[test]
fn a_running_report_sits_inset_on_a_card_like_the_picker() {
    let harness = page(Page {
        state: State::Running,
        selection: "all".to_owned(),
        part: "journalctl".to_owned(),
        step: 3,
        ..Page::default()
    });

    let picker = harness.get_by_role(Role::ComboBox).rect();
    let running = harness
        .get_by_label("Starting another collector cancels this one.")
        .rect();

    assert!(
        (running.left() - picker.left()).abs() < 0.5,
        "the running text starts at {}, the picker at {}",
        running.left(),
        picker.left()
    );
}
