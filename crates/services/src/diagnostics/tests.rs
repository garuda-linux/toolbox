use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use tokio::runtime::Runtime;
use toolbox_core::features::diagnostics::{Collector, Outcome, Selection};
use toolbox_system::DiagnosticSource;
use toolbox_system::diagnostics::page::State;
use toolbox_system::share::Upload;

use super::{Diagnostics, DryRunTransport, spawn};
use crate::testing::until;

/// Answers dmesg with a red line and the journal with nothing.
struct Fake;

impl DiagnosticSource for Fake {
    fn collect(&self, collector: Collector, _cancel: &Arc<AtomicBool>) -> Option<Outcome> {
        Some(match collector {
            Collector::Dmesg => Outcome::Report("\u{1b}[31mred\u{1b}[0m\nplain\n".to_owned()),
            Collector::Inxi
            | Collector::Health
            | Collector::SystemdAnalyze
            | Collector::Journal
            | Collector::PacmanLog => Outcome::Unavailable,
        })
    }
}

fn diagnostics(runtime: &Runtime) -> Diagnostics {
    let link = spawn(
        runtime.handle(),
        crate::waking::Wake::none(),
        Arc::new(Fake),
        Arc::new(DryRunTransport),
    );

    Diagnostics::new(Some(link))
}

fn settle(diagnostics: &mut Diagnostics, done: impl Fn(&Diagnostics) -> bool) {
    until("the service never answered", || {
        diagnostics.take();
        done(diagnostics)
    });
}

#[test]
fn a_collected_report_is_plain_text_and_coloured_lines() {
    let runtime = Runtime::new().unwrap();
    let mut diagnostics = diagnostics(&runtime);

    diagnostics.collect(Selection::One(Collector::Dmesg));
    settle(&mut diagnostics, |shown| {
        shown.page().state == State::Report
    });

    assert_eq!(diagnostics.page().report, "red\nplain\n");
    assert_eq!(diagnostics.lines().len(), 2);
    assert_eq!(diagnostics.lines()[0][0].text, "red");
}

#[test]
fn the_dry_run_uploads_nowhere_and_still_gives_a_link() {
    let runtime = Runtime::new().unwrap();
    let mut diagnostics = diagnostics(&runtime);

    assert!(!diagnostics.upload(), "an empty report is no upload");

    diagnostics.collect(Selection::One(Collector::Dmesg));
    settle(&mut diagnostics, |shown| {
        shown.page().state == State::Report
    });

    assert!(diagnostics.upload());
    assert_eq!(diagnostics.page().share, Upload::Running);
    settle(&mut diagnostics, |shown| {
        shown.page().share.paste().is_some()
    });

    let shared = diagnostics.page().share.paste();
    assert!(shared.is_some_and(|paste| {
        paste
            .link
            .starts_with("https://bin.garudalinux.org/?dryrun0000000000#")
    }));

    diagnostics.clear_upload();
    assert_eq!(diagnostics.page().share, Upload::Idle);
}

#[test]
fn a_missing_program_says_so() {
    let runtime = Runtime::new().unwrap();
    let mut diagnostics = diagnostics(&runtime);

    diagnostics.collect(Selection::One(Collector::Journal));
    settle(&mut diagnostics, |shown| {
        shown.page().state == State::Unavailable
    });

    assert_eq!(diagnostics.lines(), Vec::<Vec<toolbox_pty::Run>>::new());
}
