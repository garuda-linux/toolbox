//! One edit of a root-owned config file, as plain state.
//!
//! Read the text, run the editor on a private copy, show what changed,
//! write it back. The app does the work each step asks for, so the order
//! of things is tested here without a window, files or a helper.

mod file;
mod news;
#[cfg(test)]
mod tests;

pub use file::{EditWrite, EditedFile};
pub use news::{Changes, Compared, Fetched, Problem, Work};
use toolbox_plan::{ConfigText, EditCopy, Editor};

pub(super) const BUSY: &str = "busy";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Idle,
    /// The text is being read, maybe behind a password dialog.
    Reading,
    Starting(Stage),
    Running(Stage),
    Comparing,
    Review,
    Saved,
    Unchanged,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Editor,
    Write,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Session {
    phase: Phase,
    path: Option<EditedFile>,
    /// The file the edit is of: the path itself until the read said
    /// where its symlinks lead. The write goes there.
    target: Option<EditedFile>,
    editor: Option<Editor>,
    copy: Option<EditCopy>,
    changes: Option<Changes>,
    original: Option<ConfigText>,
    problem: Problem,
    cancelled: bool,
    editor_failed: bool,
    rereading: bool,
}

impl Phase {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Reading => "reading",
            Self::Starting(Stage::Editor) | Self::Running(Stage::Editor) => "editing",
            Self::Starting(Stage::Write) | Self::Running(Stage::Write) => "writing",
            Self::Comparing => "comparing",
            Self::Review => "review",
            Self::Saved => "saved",
            Self::Unchanged => "unchanged",
            Self::Failed => "failed",
        }
    }

    pub const fn is_free(self) -> bool {
        matches!(
            self,
            Self::Idle | Self::Saved | Self::Unchanged | Self::Failed
        )
    }
}

impl Session {
    pub const fn phase(&self) -> Phase {
        self.phase
    }

    pub const fn changes(&self) -> Option<&Changes> {
        self.changes.as_ref()
    }

    pub const fn path(&self) -> Option<&EditedFile> {
        self.path.as_ref()
    }

    pub const fn target(&self) -> Option<&EditedFile> {
        self.target.as_ref()
    }

    pub fn backup(&self) -> Option<String> {
        self.target.as_ref().map(EditedFile::backup)
    }

    pub const fn editor(&self) -> Option<Editor> {
        self.editor
    }

    pub const fn problem(&self) -> &Problem {
        &self.problem
    }

    /// # Errors
    ///
    /// `"busy"` while another edit runs or waits for review.
    pub fn start(&mut self, path: impl Into<EditedFile>) -> Result<Work, &'static str> {
        if !self.phase.is_free() {
            return Err(BUSY);
        }

        let path = path.into();
        *self = Self {
            phase: Phase::Reading,
            path: Some(path.clone()),
            target: Some(path.clone()),
            ..Self::default()
        };

        Ok(Work::Fetch(path))
    }

    pub fn fetched(&mut self, result: Result<Fetched, Problem>) -> Option<Work> {
        if self.phase != Phase::Reading {
            return result.ok().map(|fetched| Work::RemoveCopy(fetched.copy));
        }

        match result {
            Ok(fetched) => {
                self.editor = Some(fetched.editor);
                self.copy = Some(fetched.copy.clone());
                self.original = Some(fetched.original);
                self.target = Some(fetched.target);
                self.phase = Phase::Starting(Stage::Editor);
                Some(Work::RunEditor(fetched.editor, fetched.copy))
            }
            Err(problem) => {
                self.fail(problem);
                None
            }
        }
    }

    /// The apply service began a plan. Only the one this edit sent last
    /// can be meant: a plan of another page would have been refused. An
    /// edit given up before its editor started stops it now, when the
    /// cancel is sure to reach this plan and no other.
    pub fn plan_started(&mut self) -> Option<Work> {
        let Phase::Starting(stage) = self.phase else {
            return None;
        };

        self.phase = Phase::Running(stage);
        (self.cancelled && stage == Stage::Editor).then_some(Work::Cancel)
    }

    pub fn plan_refused(&mut self) -> Option<Work> {
        match self.phase {
            Phase::Starting(Stage::Editor) if self.cancelled => self.end(),
            Phase::Starting(Stage::Editor) if self.changes.is_some() => {
                self.review_with(Problem::new(BUSY, ""));
                None
            }
            Phase::Starting(Stage::Editor) => {
                self.fail(Problem::new(BUSY, ""));
                self.copy.take().map(Work::RemoveCopy)
            }
            Phase::Starting(Stage::Write) => {
                self.review_with(Problem::new(BUSY, ""));
                None
            }
            Phase::Idle
            | Phase::Reading
            | Phase::Running(_)
            | Phase::Comparing
            | Phase::Review
            | Phase::Saved
            | Phase::Unchanged
            | Phase::Failed => None,
        }
    }

    pub fn plan_finished(&mut self, succeeded: bool) -> Option<Work> {
        match self.phase {
            Phase::Running(Stage::Editor) if self.cancelled => self.end(),
            Phase::Running(Stage::Editor) => {
                self.phase = Phase::Comparing;
                self.editor_failed = !succeeded;
                self.copy
                    .clone()
                    .zip(self.original.clone())
                    .map(|(copy, original)| Work::Compare(copy, original))
            }
            Phase::Running(Stage::Write) if succeeded => {
                self.phase = Phase::Saved;
                self.changes = None;
                self.copy.take().map(Work::RemoveCopy)
            }
            Phase::Running(Stage::Write) => {
                self.review_with(Problem::new("write-failed", ""));
                None
            }
            Phase::Idle
            | Phase::Reading
            | Phase::Starting(_)
            | Phase::Comparing
            | Phase::Review
            | Phase::Saved
            | Phase::Unchanged
            | Phase::Failed => None,
        }
    }

    /// The copy to remove once the edit is over.
    pub fn compared(&mut self, result: Result<Compared, Problem>) -> Option<Work> {
        if self.phase != Phase::Comparing {
            return None;
        }

        let rereading = std::mem::take(&mut self.rereading);

        match result {
            Ok(Compared::Changed(changes)) => {
                self.original = Some(changes.original.clone());
                self.changes = Some(changes);
                self.problem = Problem::default();
                self.phase = Phase::Review;
                None
            }
            // An editor that failed without a change did not do its work,
            // which is not the same as a user who changed nothing.
            Ok(Compared::Unchanged) if self.editor_failed => {
                self.fail(Problem::new("editor-failed", ""));
                self.copy.take().map(Work::RemoveCopy)
            }
            Ok(Compared::Unchanged) => {
                self.phase = Phase::Unchanged;
                self.changes = None;
                self.copy.take().map(Work::RemoveCopy)
            }
            // The file could not be read again: the edit is still there.
            Err(problem) if rereading => {
                self.review_with(problem);
                None
            }
            Err(problem) => {
                self.fail(problem);
                self.changes = None;
                self.copy.take().map(Work::RemoveCopy)
            }
        }
    }

    pub fn apply(&mut self) -> Option<Work> {
        if self.phase != Phase::Review {
            return None;
        }

        let changes = self.changes.as_ref()?;
        let write = self
            .target
            .as_ref()?
            .write(changes.edited.clone(), changes.original.fingerprint());

        self.problem = Problem::default();
        self.phase = Phase::Starting(Stage::Write);
        Some(Work::Write(write))
    }

    pub fn edit_again(&mut self) -> Option<Work> {
        if self.phase != Phase::Review {
            return None;
        }

        let work = Work::RunEditor(self.editor?, self.copy.clone()?);

        self.problem = Problem::default();
        self.phase = Phase::Starting(Stage::Editor);
        Some(work)
    }

    pub fn reread(&mut self) -> Option<Work> {
        if self.phase != Phase::Review {
            return None;
        }

        let work = Work::Reread(self.target.clone()?, self.changes.as_ref()?.edited.clone());

        self.problem = Problem::default();
        self.rereading = true;
        self.editor_failed = false;
        self.phase = Phase::Comparing;
        Some(work)
    }

    /// The copy goes once the editor ended, as it may still write while stopped.
    pub fn discard(&mut self) -> Option<Work> {
        match self.phase {
            Phase::Starting(Stage::Editor) => {
                self.cancelled = true;
                None
            }
            Phase::Running(Stage::Editor) => {
                self.cancelled = true;
                Some(Work::Cancel)
            }
            Phase::Review => self.end(),
            Phase::Idle
            | Phase::Reading
            | Phase::Starting(Stage::Write)
            | Phase::Running(Stage::Write)
            | Phase::Comparing
            | Phase::Saved
            | Phase::Unchanged
            | Phase::Failed => None,
        }
    }

    pub fn dismiss(&mut self) {
        if matches!(self.phase, Phase::Saved | Phase::Unchanged | Phase::Failed) {
            *self = Self::default();
        }
    }

    fn end(&mut self) -> Option<Work> {
        let copy = self.copy.take();

        *self = Self::default();
        copy.map(Work::RemoveCopy)
    }

    fn fail(&mut self, problem: Problem) {
        self.phase = Phase::Failed;
        self.problem = problem;
    }

    fn review_with(&mut self, problem: Problem) {
        self.phase = Phase::Review;
        self.problem = problem;
    }
}
