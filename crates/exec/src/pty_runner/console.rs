//! The AUR steps that need this runner's own state: the build as the user,
//! and the Console of a review. A child of the runner, so it reads its
//! answers, home and stop flag directly.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::RecvTimeoutError;
use std::time::SystemTime;
use std::{io, thread};

use toolbox_plan::protocol::{Answer, Outcome};
use toolbox_plan::{BuildRun, PackageName, StepId};
use toolbox_pty::TerminalSize;

use super::{POLL_INTERVAL, PtyRunner};
use crate::apply::EventSink;
use crate::aur::review::Console;
use crate::capture::{CaptureError, Captured, Policy, capture};
use crate::report::failure;
use crate::{Environment, aur};

/// Input this long without a line break is no yes, so the wait ends with it.
const LINE_LIMIT: usize = 8;

impl PtyRunner {
    /// `makepkg` of an AUR build as the user. Only the app has a home to
    /// build in, so the helper fails here before anything starts.
    pub(super) fn build_aur(
        &mut self,
        step: StepId,
        (makepkg, git): (&Path, &Path),
        (base, run): (&PackageName, &BuildRun),
        sink: &mut dyn EventSink,
    ) -> Outcome {
        if self.cancelled_before() {
            return Outcome::Cancelled;
        }

        let Some(home) = self.home.as_deref() else {
            return failure(
                step,
                sink,
                "an AUR build runs as the user, and this process has no home",
            );
        };

        let cancel = self.stop.clone().unwrap_or_default();
        match aur::build::prepare(
            home,
            (makepkg, git),
            (base, run),
            SystemTime::now(),
            &cancel,
        ) {
            Ok(pty) => self.spawn_pty(step, &"makepkg", &pty, sink),
            Err(error) => failure(step, sink, &error.to_string()),
        }
    }
}

impl Console for PtyRunner {
    fn home(&self) -> Option<PathBuf> {
        self.home.clone()
    }

    fn stop(&self) -> Arc<AtomicBool> {
        self.stop.clone().unwrap_or_default()
    }

    /// The capture runs on a thread of its own, so the answers are read
    /// while paru runs. paru runs in the user's environment, since it finds
    /// its config and cache through `HOME` and `XDG_*`.
    fn watched(
        &mut self,
        mut command: Command,
        policy: Policy<'static>,
    ) -> Result<Captured, CaptureError> {
        if self.cancelled_before() {
            return Err(CaptureError::Cancelled);
        }

        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker = thread::Builder::new()
            .name("review-capture".to_owned())
            .spawn(move || {
                capture(
                    &mut command,
                    Environment::Inherited,
                    &[],
                    &policy,
                    &worker_stop,
                )
            })
            .map_err(CaptureError::Failed)?;

        while !worker.is_finished() {
            if self.stop_requested() {
                stop.store(true, Ordering::Relaxed);
            }

            match self.answers.recv_timeout(POLL_INTERVAL) {
                Ok(Answer::Cancel | Answer::Finish) | Err(RecvTimeoutError::Disconnected) => {
                    stop.store(true, Ordering::Relaxed);
                    self.cancel(None);
                }
                Ok(Answer::Resize { size }) => {
                    self.size = TerminalSize::from((size.columns, size.rows));
                }
                Ok(Answer::Input { data: _ } | Answer::Proceed { step: _ })
                | Err(RecvTimeoutError::Timeout) => {}
            }
        }

        worker.join().unwrap_or_else(|_| {
            Err(CaptureError::Failed(io::Error::other(
                "the capture of paru panicked",
            )))
        })
    }

    fn drain(&mut self) {
        self.take_answers(None);
    }

    /// Input is read here, since no program runs to take it. The line ends at
    /// the first line break, so a yes must be `y` and Enter, nothing else.
    fn line(&mut self) -> Option<String> {
        let mut typed = String::new();

        loop {
            if self.stop_requested() {
                tracing::info!("asked to stop by a signal");
                self.cancel(None);
            }

            if self.cancelled || self.finishing {
                return None;
            }

            match self.answers.recv_timeout(POLL_INTERVAL) {
                Ok(Answer::Input { data }) => {
                    typed.push_str(&data);

                    if let Some(end) = typed.find(['\r', '\n']) {
                        typed.truncate(end);
                        return Some(typed);
                    }

                    if typed.len() > LINE_LIMIT {
                        return Some(typed);
                    }
                }
                Ok(Answer::Resize { size }) => {
                    self.size = TerminalSize::from((size.columns, size.rows));
                }
                Ok(Answer::Cancel) | Err(RecvTimeoutError::Disconnected) => {
                    self.cancel(None);
                    return None;
                }
                Ok(Answer::Finish) => self.finishing = true,
                Ok(Answer::Proceed { step: _ }) => {
                    tracing::warn!("ignoring a proceed that no gate waits for");
                }
                Err(RecvTimeoutError::Timeout) => {}
            }
        }
    }
}
