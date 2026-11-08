//! `garuda-toolbox diagnostics`: the page's collectors and upload, the
//! report on standard output.

#[cfg(test)]
mod tests;

use std::io::Write;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

use tokio::runtime::Handle;
use toolbox_cli::Exit;
use toolbox_cli::confirm::{Prompt, consent};
use toolbox_core::features::diagnostics::Selection;
use toolbox_privatebin::Transport;
use toolbox_system::DiagnosticSource;
use toolbox_system::diagnostics::page::{Page, State};
use toolbox_system::diagnostics::service::{self, DiagnosticsNote, DiagnosticsSink, Request};
use toolbox_system::share::{Upload, UploadProblem};

struct Notes(parking_lot::Mutex<Sender<DiagnosticsNote>>);

impl DiagnosticsSink for Notes {
    fn note(&self, note: DiagnosticsNote) {
        // The command stopped listening: nothing is left to tell.
        self.0.lock().send(note).ok();
    }
}

/// Collects `selection`, prints it, and uploads it after the question
/// when `upload` asks for it.
#[expect(
    clippy::too_many_arguments,
    reason = "the ports and the terminal, each a seam the tests use"
)]
pub fn collect(
    selection: Selection,
    upload: bool,
    yes: bool,
    runtime: &Handle,
    source: Arc<dyn DiagnosticSource>,
    transport: Arc<dyn Transport>,
    prompt: &mut dyn Prompt,
    out: &mut dyn Write,
) -> Exit {
    let (sender, notes) = channel();
    let handle = service::spawn(
        runtime,
        source,
        transport,
        Arc::new(Notes(parking_lot::Mutex::new(sender))),
    );

    let mut page = Page::default();

    handle.send(Request::Collect(selection));
    while let Ok(note) = notes.recv() {
        let finished = matches!(note, DiagnosticsNote::Finished(..));
        page.apply(note);
        if finished {
            break;
        }
    }

    let exit = match page.state {
        State::Report => Exit::Success,
        State::Unavailable => Exit::Unavailable,
        State::Refused(_) => Exit::Refused,
        State::Idle | State::Running | State::Failed => Exit::Failure,
    };

    write!(out, "{}", page.report).ok();
    if !page.report.ends_with('\n') {
        writeln!(out).ok();
    }

    if exit != Exit::Success || !upload {
        return exit;
    }

    let paint = crate::paint();
    writeln!(out, "\n{}\n", paint.title("Upload", None)).ok();
    let note = "The report goes to bin.garudalinux.org as an encrypted paste. Whoever has the link can read it.";
    writeln!(out, "{}", paint.item(note)).ok();
    if let Some(refused) = consent(prompt, yes, None).exit() {
        return refused;
    }

    handle.send(Request::Upload(page.report.clone()));
    uploaded(&mut page, &notes, out)
}

fn uploaded(page: &mut Page, notes: &Receiver<DiagnosticsNote>, out: &mut dyn Write) -> Exit {
    while let Ok(note) = notes.recv() {
        let done = matches!(note, DiagnosticsNote::Uploaded(_));
        page.apply(note);
        if done {
            break;
        }
    }

    let stopped = UploadProblem::stopped();
    let problem = match &page.share {
        Upload::Done(paste) => {
            let facts = [
                ("link", paste.link.as_str()),
                ("delete", paste.delete_link.as_str()),
            ];

            write!(out, "\n{}", crate::paint().facts("Uploaded", &facts)).ok();
            return Exit::Success;
        }
        Upload::Failed(problem) => problem,
        // The service went away before it told how the upload ended.
        Upload::Idle | Upload::Running => &stopped,
    };

    let what = format!("the upload failed ({})", problem.kind.as_str());
    crate::out::failure(out, &what, &problem.detail);
    Exit::Failure
}
