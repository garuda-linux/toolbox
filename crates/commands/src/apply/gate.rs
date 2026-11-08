//! When a line typed in the terminal may reach the running program. No
//! step runs while pkexec asks for the password, so the gate is shut then
//! and nothing typed for pkexec can be taken for the program.

#[cfg(test)]
mod tests;

use parking_lot::Mutex;
use toolbox_plan::protocol::Event;
use toolbox_services::apply::Report;

/// The end of a program's output kept to spot a question, enough for any
/// prompt line.
const TAIL: usize = 256;

#[derive(Debug, Default)]
pub struct Gate(Mutex<State>);

#[derive(Debug, Default)]
struct State {
    running: bool,
    done: bool,
    /// The output since the last line break.
    tail: String,
}

impl Gate {
    pub fn note(&self, report: &Report) {
        let mut state = self.0.lock();

        match report {
            Report::Step(Event::Started { step: _ }) => {
                state.running = true;
                state.tail.clear();
            }
            Report::Step(Event::Finished {
                step: _,
                outcome: _,
            }) => state.running = false,
            Report::Step(Event::Output { step: _, data }) => state.follow(data),
            Report::PlanFinished { succeeded: _ } => {
                state.running = false;
                state.done = true;
            }
            Report::PlanStarted { steps: _ }
            | Report::Step(_)
            | Report::Problem(_)
            | Report::Busy
            | Report::CancelWaits => {}
        }
    }

    /// A step runs, so a typed line is meant for it.
    pub fn open(&self) -> bool {
        self.0.lock().running
    }

    /// The running step asks for a password, so the line goes without echo.
    pub fn secret(&self) -> bool {
        let state = self.0.lock();
        state.running && asks_secret(&state.tail)
    }

    pub fn done(&self) -> bool {
        self.0.lock().done
    }
}

impl State {
    fn follow(&mut self, data: &str) {
        match data.rfind('\n') {
            Some(at) => {
                self.tail.clear();
                self.tail.push_str(data.get(at + 1..).unwrap_or_default());
            }
            None => self.tail.push_str(data),
        }

        if self.tail.len() > TAIL {
            let cut = self.tail.len() - TAIL;
            let start = (cut..self.tail.len())
                .find(|at| self.tail.is_char_boundary(*at))
                .unwrap_or(self.tail.len());

            self.tail.drain(..start);
        }
    }
}

pub fn asks_secret(tail: &str) -> bool {
    let line = tail.trim_end().to_lowercase();
    let asks = line.ends_with(':') || line.ends_with('?');
    let secret = ["password", "passphrase", "passwort", "kennwort"]
        .iter()
        .any(|word| line.contains(word));

    asks && secret
}
