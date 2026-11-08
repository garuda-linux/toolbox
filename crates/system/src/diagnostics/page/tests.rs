use toolbox_privatebin::Paste;
use toolbox_state::diagnostics::{Collector, Outcome, Refusal, Selection, UploadFailure};

use super::super::service::DiagnosticsNote;
use super::{Changed, Page, State};
use crate::share::{Upload, UploadProblem};

const DMESG: Selection = Selection::One(Collector::Dmesg);

fn finished(outcome: Outcome) -> DiagnosticsNote {
    DiagnosticsNote::Finished(DMESG, outcome)
}

fn offline() -> UploadProblem {
    UploadProblem {
        kind: UploadFailure::Network,
        detail: "no network".to_owned(),
    }
}

fn with_report() -> Page {
    let mut page = Page::default();

    page.apply(DiagnosticsNote::Started(DMESG));
    page.apply(finished(Outcome::Report("kernel log\n".to_owned())));
    page
}

#[test]
fn a_fresh_page_is_idle_without_text_or_upload() {
    let page = Page::default();

    assert_eq!(page.state.as_str(), "idle");
    assert_eq!(page.share, Upload::Idle);
    assert_eq!((page.selection.as_str(), page.report.as_str()), ("", ""));
}

#[test]
fn a_start_shows_the_run_and_takes_the_old_report_away() {
    let mut page = with_report();

    assert_eq!(
        page.apply(DiagnosticsNote::Started(Selection::All)),
        Changed::Report { ready: false }
    );
    assert_eq!(page.state.as_str(), "running");
    assert_eq!(page.selection, "all");
    assert_eq!(page.report, "");
}

#[test]
fn a_report_is_ready_for_the_clipboard() {
    let mut page = Page::default();

    page.apply(DiagnosticsNote::Started(DMESG));

    assert_eq!(
        page.apply(finished(Outcome::Report("kernel log\n".to_owned()))),
        Changed::Report { ready: true }
    );
    assert_eq!(page.state.as_str(), "report");
    assert_eq!(page.selection, "dmesg");
    assert_eq!(page.report, "kernel log\n");
    assert_eq!(page.problem, "");
}

#[test]
fn an_empty_report_is_still_a_report() {
    let mut page = Page::default();

    page.apply(finished(Outcome::Report(String::new())));

    assert_eq!(page.state, State::Report);
}

#[test]
fn the_other_outcomes_are_states_without_text() {
    for (outcome, state, problem) in [
        (Outcome::Unavailable, "unavailable", ""),
        (Outcome::Refused(Refusal::Dismissed), "dismissed", ""),
        (Outcome::Refused(Refusal::Denied), "denied", ""),
        (Outcome::Refused(Refusal::NoAgent), "no-agent", ""),
        (
            Outcome::Refused(Refusal::HelperMissing),
            "helper-missing",
            "",
        ),
        (
            Outcome::Failed("dmesg: Operation not permitted".to_owned()),
            "failed",
            "dmesg: Operation not permitted",
        ),
    ] {
        let mut page = with_report();

        assert_eq!(
            page.apply(finished(outcome)),
            Changed::Report { ready: false }
        );
        assert_eq!(page.state.as_str(), state);
        assert_eq!(page.problem, problem);
        assert_eq!(page.report, "", "{state} kept the old report");
    }
}

#[test]
fn a_full_report_counts_its_collectors_from_one() {
    let mut page = Page::default();

    page.apply(DiagnosticsNote::Started(Selection::All));

    assert_eq!(
        page.apply(DiagnosticsNote::Collecting(Collector::Inxi)),
        Changed::Status
    );
    assert_eq!((page.part.as_str(), page.step), ("inxi", 1_i32));

    page.apply(DiagnosticsNote::Collecting(Collector::Health));
    assert_eq!((page.part.as_str(), page.step), ("garuda-health", 6_i32));

    page.apply(DiagnosticsNote::Finished(
        Selection::All,
        Outcome::Report("full\n".to_owned()),
    ));
    assert_eq!((page.part.as_str(), page.step), ("", 0_i32));
}

#[test]
fn a_cancel_returns_to_idle() {
    let mut page = Page::default();

    page.apply(DiagnosticsNote::Started(Selection::All));
    page.apply(DiagnosticsNote::Collecting(Collector::Journal));

    assert_eq!(page.apply(DiagnosticsNote::Cancelled), Changed::Status);
    assert_eq!(page.state, State::Idle);
    assert_eq!((page.part.as_str(), page.step), ("", 0_i32));
}

#[test]
fn an_upload_keeps_the_report_and_ends_with_a_link_or_a_reason() {
    let mut page = with_report();

    assert_eq!(
        page.apply(DiagnosticsNote::UploadStarted),
        Changed::Upload { done: false }
    );
    assert_eq!(page.share, Upload::Running);

    let paste = Paste {
        link: "https://bin.garudalinux.org/?abc#key".to_owned(),
        delete_link: "https://bin.garudalinux.org/?pasteid=abc&deletetoken=token".to_owned(),
    };

    assert_eq!(
        page.apply(DiagnosticsNote::Uploaded(Ok(paste.clone()))),
        Changed::Upload { done: true }
    );
    assert_eq!(page.share, Upload::Done(paste.clone()));
    assert_eq!(page.share.paste(), Some(&paste));
    assert_eq!(page.report, "kernel log\n");

    page.apply(DiagnosticsNote::UploadStarted);
    assert_eq!(page.share.paste(), None, "a new upload kept the old link");

    assert_eq!(
        page.apply(DiagnosticsNote::Uploaded(Err(offline()))),
        Changed::Upload { done: false }
    );
    assert_eq!(page.share, Upload::Failed(offline()));

    page.clear_upload();
    assert_eq!(page.share, Upload::Idle);
}

#[test]
fn a_new_collection_keeps_the_last_upload() {
    let mut page = with_report();

    let paste = Paste {
        link: "https://bin.garudalinux.org/?abc#key".to_owned(),
        delete_link: String::new(),
    };

    page.apply(DiagnosticsNote::Uploaded(Ok(paste.clone())));
    page.apply(DiagnosticsNote::Started(Selection::All));

    assert_eq!(page.share, Upload::Done(paste));
}

#[test]
fn a_coloured_report_keeps_its_colours_to_show_and_a_plain_form_for_copy_and_upload() {
    let mut page = Page::default();
    let printed = "\u{1b}[31m<b>failed</b>\u{1b}[0m & done\n";

    page.apply(finished(Outcome::Report(printed.to_owned())));

    assert_eq!(page.report, "<b>failed</b> & done\n");
    assert_eq!(page.raw, printed);
}

#[test]
fn a_start_and_the_other_outcomes_take_the_report_away() {
    for note in [
        DiagnosticsNote::Started(DMESG),
        DiagnosticsNote::Cancelled,
        finished(Outcome::Unavailable),
    ] {
        let mut page = Page::default();

        page.apply(finished(Outcome::Report(
            "\u{1b}[32mok\u{1b}[0m\n".to_owned(),
        )));
        page.apply(note);

        assert_eq!((page.report.as_str(), page.raw.as_str()), ("", ""));
    }
}
