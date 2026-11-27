#[cfg(test)]
mod tests;

use toolbox_plan::ConfigFilePath;
use toolbox_text::OutsideText;

use super::Refusal;
use super::session::Problem;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Idle,
    /// The text is being read, maybe behind a password dialog.
    Reading,
    Shown,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Viewed {
    /// Where the text was read: the path, or where its links lead.
    pub path: ConfigFilePath,
    pub content: OutsideText,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Viewing {
    phase: Phase,
    path: Option<ConfigFilePath>,
    read: Option<Viewed>,
    problem: Option<Problem>,
    /// The number of the last read asked for. A read that runs on after
    /// its dialog closed must not fill the next one.
    asked: u64,
}

impl Phase {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Reading => "reading",
            Self::Shown => "shown",
            Self::Failed => "failed",
        }
    }
}

impl Viewing {
    pub const fn phase(&self) -> Phase {
        self.phase
    }

    pub const fn path(&self) -> Option<&ConfigFilePath> {
        self.path.as_ref()
    }

    pub const fn viewed(&self) -> Option<&Viewed> {
        self.read.as_ref()
    }

    pub const fn problem(&self) -> Option<&Problem> {
        self.problem.as_ref()
    }

    /// # Errors
    ///
    /// [`Refusal::Busy`] while a read runs, so a second click asks for no
    /// second password.
    pub fn start(&mut self, path: ConfigFilePath) -> Result<u64, Refusal> {
        if self.phase == Phase::Reading {
            return Err(Refusal::Busy);
        }

        let asked = self.asked.wrapping_add(1);
        *self = Self {
            phase: Phase::Reading,
            path: Some(path),
            asked,
            ..Self::default()
        };

        Ok(asked)
    }

    /// An answer for a dialog that closed is not shown.
    pub fn read(&mut self, number: u64, result: Result<Viewed, Problem>) {
        if self.phase != Phase::Reading || number != self.asked {
            return;
        }

        match result {
            Ok(read) => {
                self.phase = Phase::Shown;
                self.read = Some(read);
            }
            Err(problem) => {
                self.phase = Phase::Failed;
                self.problem = Some(problem);
            }
        }
    }

    pub fn close(&mut self) {
        *self = Self {
            asked: self.asked,
            ..Self::default()
        };
    }
}
