mod heights;
mod redacted;
mod scroll;
mod shown;
mod viewer;

use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use parking_lot::Mutex;
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
use crate::links::{Opener, install};
use crate::page_service::Services;
use crate::testing::{PageFixture, PageState, button_kind};

type Shown = Harness<'static, PageState<DiagnosticsPage>>;

#[derive(Debug, Default)]
struct Opened(Mutex<Vec<String>>);

impl Opener for Opened {
    fn open(&self, url: &str) -> bool {
        self.0.lock().push(url.to_owned());
        true
    }
}

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
        selection: Some(Selection::One(Collector::Dmesg)),
        report: plain.to_owned(),
        shareable: Some(redact(plain, &Known::default())),
        raw: "kernel: hello\n\u{1b}[31mkernel: oops\u{1b}[0m\n".to_owned(),
        ..Page::default()
    }
}

fn plain_report(text: String) -> Page {
    Page {
        state: State::Report,
        selection: Some(Selection::One(Collector::Dmesg)),
        report: text.clone(),
        raw: text,
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
fn every_report_has_its_row_and_collects_from_it() {
    let mut harness = page(Page::default());

    assert!(harness.query_by_role(Role::ComboBox).is_none());
    assert_eq!(harness.query_all_by_label("Collect").count(), 0);
    assert!(harness.query_by_label("Full report").is_some());

    harness.get_by_label("Collect Kernel log").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(DiagnosticsCommand::Collect(Selection::One(
            Collector::Dmesg
        )))]
    );
}

#[test]
fn a_running_full_report_names_its_part_and_its_row_cancels_it() {
    let mut harness = page(Page {
        state: State::Running,
        selection: Some(Selection::All),
        part: Some(Collector::Journal),
        step: 3,
        ..Page::default()
    });

    assert!(
        harness
            .query_by_label_contains("Collecting Journal (3 of 6)")
            .is_some()
    );
    assert!(harness.query_by_label("Collect Full report").is_none());
    assert!(harness.query_by_label("Collect Journal").is_some());

    harness.get_by_label("Cancel").click();
    harness.run();

    assert_eq!(sent(&harness), [Command::from(DiagnosticsCommand::Cancel)]);
}

#[test]
fn a_report_shows_its_lines_and_is_shared_after_the_question() {
    let mut harness = page(report());

    assert!(harness.query_by_label("kernel: oops").is_some());

    harness.get_by_label("Share Report…").click();
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Upload This Report?")
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

    harness.get_by_label("Copy Report").click();
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
        selection: Some(Selection::One(Collector::Dmesg)),
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
fn an_upload_is_one_row_with_the_delete_link_in_its_menu() {
    let mut harness = page(uploaded());

    assert!(harness.query_by_label("Uploaded.").is_some());
    assert!(harness.query_by_label("Delete Now…").is_none(), "collapsed");

    // Here and not in the app's snapshots: a real upload's key differs
    // each time.
    harness.snapshot("diagnostics/uploaded_dark");

    harness.get_by_label("More link actions").click();
    harness.run();
    harness.get_by_label("Delete Now…").click();
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Delete the Report from the Server?")
            .is_some()
    );

    harness.get_by_label("Cancel").click();
    harness.run();

    assert!(harness.query_by_role(Role::Dialog).is_none());
    assert_eq!(sent(&harness), []);

    let fake: Arc<dyn Opener> = Arc::new(Opened::default());
    install(&harness.ctx, fake);
    harness.get_by_label("More link actions").click();
    harness.run();
    harness.get_by_label("Delete Now…").click();
    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Delete")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(DiagnosticsCommand::HideShare)]
    );
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
fn each_row_says_what_its_report_holds() {
    let harness = page(Page::default());

    assert!(
        harness
            .query_by_label("Hardware, drivers and system overview from garuda-inxi.")
            .is_some()
    );
    assert!(
        harness
            .query_by_label("All of the above in one report. May ask for your password twice.")
            .is_some()
    );
}

#[test]
fn copy_report_leads_until_an_upload_then_its_link_does() {
    let harness = page(report());
    assert_eq!(button_kind(&harness, "Copy Report"), ButtonKind::Primary);

    let shared = page(uploaded());

    assert_eq!(button_kind(&shared, "Copy Report"), ButtonKind::Soft);
    assert_eq!(button_kind(&shared, "Copy Link"), ButtonKind::Primary);
}

#[test]
fn the_text_to_upload_sits_in_a_frame_of_its_own() {
    let mut harness = page(report());
    harness.get_by_label("Share Report…").click();
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
