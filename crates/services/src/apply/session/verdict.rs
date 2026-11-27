//! What the helper's exit says when the session closes: a refused or broken
//! pkexec, a helper ended without `Done`, or the plan's result.

use std::process::ExitStatus;

use tokio::process::Child;
use tokio::task::JoinHandle;
use toolbox_core::features::console::HelperFailure;
use toolbox_system::pkexec::{self, Refusal};

use super::super::handle::Reporter;
use super::super::{Launcher, Problem, Report};
use super::Heard;

/// The launcher that may explain a refusal, and the words of its stderr.
pub(super) struct Complaints<'launcher> {
    pub(super) launcher: &'launcher dyn Launcher,
    pub(super) words: Option<JoinHandle<String>>,
}

pub(super) async fn finish(
    child: &mut Child,
    steps_ok: bool,
    heard: Heard,
    complaints: Complaints<'_>,
    reporter: &Reporter,
) -> bool {
    let status = match child.wait().await {
        Ok(status) => status,
        Err(error) => {
            reporter.helper_failed(HelperFailure::Lost, error.to_string());
            return false;
        }
    };

    if heard == Heard::Nothing
        && let Some((refusal, words)) = refusal(status, complaints).await
    {
        if refusal == Refusal::Broken {
            let detail = pkexec::reason(&words).to_owned();
            reporter.helper_failed(HelperFailure::Pkexec, detail);
            return false;
        }

        if refusal.not_authorized() {
            tracing::info!("the plan did not start: {refusal}");
        } else {
            tracing::error!("the plan did not start: {refusal}");
        }

        reporter.report(Report::Problem(Problem::Refused(refusal)));
        return false;
    }

    // Killed or crashed: without this the plan would fail without a word.
    if heard != Heard::Done {
        reporter.helper_failed(HelperFailure::Ended, status.to_string());
        return false;
    }

    steps_ok && status.success()
}

/// Why pkexec or sudo did not run the helper, when its exit code says so. Its
/// words are waited for only then: pkexec has ended and nothing it
/// started holds the pipe.
async fn refusal(status: ExitStatus, complaints: Complaints<'_>) -> Option<(Refusal, String)> {
    let Complaints { launcher, words } = complaints;
    if !launcher.may_refuse(status.code()) {
        return None;
    }

    let words = match words {
        Some(reading) => reading.await.unwrap_or_else(|error| {
            tracing::warn!("helper's error text lost: {error}");

            String::new()
        }),
        None => String::new(),
    };

    launcher
        .refusal(status.code(), &words)
        .map(|refusal| (refusal, words))
}
