//! Lines typed in the terminal go to the running step, so paru can ask its
//! questions. Only from a terminal, and only while a step runs: pkexec asks
//! for the password before any step, so nothing typed for it is read here.

use std::io::{BufRead as _, IsTerminal as _};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use rustix::event::{PollFd, PollFlags, Timespec, poll};
use toolbox_services::apply::ApplyCommand;

use super::{LOOK_AGAIN, Running};
use crate::terminal::{self, Typed};

pub fn forward_input(run: &Arc<Running>) -> Option<JoinHandle<()>> {
    if !std::io::stdin().is_terminal() {
        return None;
    }

    let run = Arc::clone(run);

    let started = std::thread::Builder::new()
        .name("cli-input".to_owned())
        .spawn(move || type_into(&run));

    started
        .inspect_err(|error| tracing::warn!("typed answers cannot reach the program: {error}"))
        .ok()
}

fn type_into(run: &Running) {
    let gate = &run.gate;

    while !gate.done() {
        if !gate.open() {
            std::thread::sleep(LOOK_AGAIN);
            continue;
        }

        if gate.secret() {
            match terminal::hidden_line(&|| gate.secret()) {
                Some(Typed::Line(mut line)) => {
                    line.push('\n');
                    run.service.send(ApplyCommand::Input(line.into_bytes()));
                }
                // A step that waits for a password cannot end on its own,
                // so the key stops it at once instead of after it.
                Some(Typed::Cancelled) => run.service.send(ApplyCommand::Cancel),
                Some(Typed::Closed) | None => std::thread::sleep(LOOK_AGAIN),
            }

            continue;
        }

        if typed_ahead(LOOK_AGAIN) && gate.open() {
            let mut line = String::new();
            if std::io::stdin()
                .lock()
                .read_line(&mut line)
                .is_ok_and(|read| read > 0)
            {
                run.service.send(ApplyCommand::Input(line.into_bytes()));
            }
        }
    }
}

fn typed_ahead(wait: Duration) -> bool {
    let stdin = std::io::stdin();
    let mut fds = [PollFd::new(&stdin, PollFlags::IN)];
    let timeout = Timespec::try_from(wait).ok();

    poll(&mut fds, timeout.as_ref()).is_ok_and(|ready| ready > 0)
}
