use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use toolbox_cli::Exit;
use toolbox_cli::confirm::Prompt;
use toolbox_core::features::diagnostics::{Collector, Outcome, Selection};
use toolbox_services::diagnostics::DryRunTransport;
use toolbox_system::DiagnosticSource;
use toolbox_system::diagnostics::page::Page;
use toolbox_system::diagnostics::service::DiagnosticsNote;

use super::collect;

struct Fixed(Outcome);

impl DiagnosticSource for Fixed {
    fn collect(&self, _collector: Collector, _cancel: &Arc<AtomicBool>) -> Option<Outcome> {
        Some(self.0.clone())
    }
}

struct Answers(VecDeque<&'static str>);

impl Prompt for Answers {
    fn is_terminal(&self) -> bool {
        true
    }

    fn ask(&mut self, _question: &str) -> Option<String> {
        self.0.pop_front().map(str::to_owned)
    }
}

fn run(outcome: Outcome, upload: bool, answers: &[&'static str]) -> (Exit, String) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut out = Vec::new();
    let exit = collect(
        Selection::One(Collector::Inxi),
        upload,
        false,
        runtime.handle(),
        Arc::new(Fixed(outcome)),
        Arc::new(DryRunTransport),
        &mut Answers(answers.iter().copied().collect()),
        &mut out,
    );

    (exit, String::from_utf8(out).unwrap())
}

#[test]
fn the_report_is_printed_as_the_page_shows_it() {
    let (exit, out) = run(Outcome::Report("CPU: fast".to_owned()), false, &[]);

    assert_eq!(exit, Exit::Success);
    assert!(out.contains("CPU: fast"), "{out}");
}

#[test]
fn a_missing_program_is_unavailable() {
    let (exit, _) = run(Outcome::Unavailable, false, &[]);

    assert_eq!(exit, Exit::Unavailable);
}

#[test]
fn an_upload_asks_first_and_prints_both_links() {
    let (exit, out) = run(Outcome::Report("CPU: fast".to_owned()), true, &["y"]);

    assert_eq!(exit, Exit::Success);
    assert!(
        out.contains("\nUpload\n\n  The report goes to bin.garudalinux.org"),
        "{out}"
    );
    assert!(out.contains("\n\nUploaded\n\n  link    "), "{out}");
    assert!(out.contains("\n  delete  "), "{out}");
}

#[test]
fn a_declined_upload_keeps_the_report_local() {
    let (exit, out) = run(Outcome::Report("CPU: fast".to_owned()), true, &["n"]);

    assert_eq!(exit, Exit::Declined);
    assert!(!out.contains("Uploaded"), "{out}");
}

#[test]
fn an_upload_that_never_ends_is_a_failure_without_links() {
    let (notes, heard) = std::sync::mpsc::channel();
    notes.send(DiagnosticsNote::UploadStarted).unwrap();
    drop(notes);

    let mut out = Vec::new();
    let exit = super::uploaded(&mut Page::default(), &heard, &mut out);
    let out = String::from_utf8(out).unwrap();

    assert_eq!(exit, Exit::Failure);
    assert!(!out.contains("Uploaded"), "{out}");
}
