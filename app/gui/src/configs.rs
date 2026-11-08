//! The Config Files page's own state: which files exist, the edit of a
//! file and the view of a read-only one.
//!
//! Core's [`Session`] and [`Viewing`] decide every step. This part carries
//! out what a step asks for: work on a blocking thread, or a plan the app
//! runs in the Console. No editor ever runs as root.

pub(crate) mod jobs;
#[cfg(test)]
mod tests;
mod turns;

use toolbox_catalog::{ConfigFileCatalog, LoadProblem};
use toolbox_core::features::configs::session::{EditedFile, Phase, Session, Work};
use toolbox_core::features::configs::viewing::Viewing;
use toolbox_core::features::configs::{
    Listing, Target, edit_plan, target as target_of, view_target, write_plan,
};
use toolbox_plan::Plan;

pub use self::jobs::ConfigsSetup;
use self::jobs::{Jobs, Landed};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    /// Runs in the Console: the editor, or the write as root.
    Run(Plan),
    Cancel,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Taken {
    pub acts: Vec<Act>,
    /// The edit needs the user on the page again: changes to review, or
    /// how it ended.
    pub review: bool,
}

#[derive(Debug)]
pub struct Configs {
    catalog: Result<ConfigFileCatalog, LoadProblem>,
    listing: Option<Listing>,
    /// A listing is on its way. The page opens every frame, and a second
    /// read would make the first one's answer stale before it lands.
    listing_asked: bool,
    session: Session,
    viewing: Viewing,
    /// The phase the last [`Configs::take`] saw, so every step that brings
    /// an edit to the user is told once, whatever made it.
    seen: Phase,
    /// `None` in tests that need no reads: nothing is read.
    jobs: Option<Jobs>,
}

impl Default for Configs {
    fn default() -> Self {
        Self {
            catalog: Ok(ConfigFileCatalog::default()),
            listing: None,
            listing_asked: false,
            session: Session::default(),
            viewing: Viewing::default(),
            seen: Phase::Idle,
            jobs: None,
        }
    }
}

impl Configs {
    pub fn new(setup: Option<ConfigsSetup>, ctx: egui::Context) -> Self {
        let Some(setup) = setup else {
            return Self::default();
        };

        let (jobs, catalog) = Jobs::new(setup, ctx);
        Self {
            catalog,
            jobs: Some(jobs),
            ..Self::default()
        }
    }

    pub const fn catalog(&self) -> &Result<ConfigFileCatalog, LoadProblem> {
        &self.catalog
    }

    /// The files of the catalog that exist, `None` before the first look.
    pub const fn listing(&self) -> Option<&Listing> {
        self.listing.as_ref()
    }

    pub const fn session(&self) -> &Session {
        &self.session
    }

    pub const fn viewing(&self) -> &Viewing {
        &self.viewing
    }

    /// The page shows: its first look lists the files.
    pub fn open(&mut self) {
        if self.listing.is_none() && !self.listing_asked {
            self.list();
        }
    }

    pub fn refresh(&mut self) {
        self.list();
    }

    /// Opens the file of the user behind the row `key` in the desktop's
    /// app for it.
    ///
    /// # Errors
    ///
    /// `"unknown"` for a row that is no file of the user, or one that is
    /// gone.
    pub fn open_home(&self, key: &str) -> Result<(), &'static str> {
        let target = self.target(key).ok_or("unknown")?;
        let jobs = self.jobs.as_ref().ok_or("unknown")?;
        jobs.open_home(&target).then_some(()).ok_or("unknown")
    }

    /// Starts the edit of the file behind the row `key`.
    ///
    /// # Errors
    ///
    /// `"busy"` while another edit is open, `"unknown"` for a row that is
    /// no file, `"read-only"` for a file the catalog only shows.
    pub fn edit(&mut self, key: &str) -> Result<Option<Act>, &'static str> {
        let file: EditedFile = match self.target(key).ok_or("unknown")? {
            // The row offers no edit. A caller that asks anyway learns why
            // before the user edits a copy the helper would never write.
            Target::Root(path) if !path.is_writable() => return Err("read-only"),
            Target::Root(path) => path.into(),
            Target::Home(path) => path.into(),
        };

        let work = self.session.start(file)?;
        Ok(self.perform(work))
    }

    /// Reads the read-only file behind the row `key` for the view.
    ///
    /// # Errors
    ///
    /// `"unknown"` for a row with nothing to view, `"busy"` while a read
    /// for the view runs.
    pub fn view(&mut self, key: &str) -> Result<(), &'static str> {
        let (Ok(catalog), Some(listing)) = (&self.catalog, &self.listing) else {
            return Err("unknown");
        };

        let path = view_target(catalog, listing, key).ok_or("unknown")?;
        let number = self.viewing.start(path.clone())?;
        if let Some(jobs) = &self.jobs {
            jobs.view(number, path);
        }

        Ok(())
    }

    /// The view closed. The text goes with it.
    pub fn close_view(&mut self) {
        self.viewing.close();
    }

    pub fn apply(&mut self) -> Option<Act> {
        let work = self.session.apply()?;
        self.perform(work)
    }

    pub fn edit_again(&mut self) -> Option<Act> {
        let work = self.session.edit_again()?;
        self.perform(work)
    }

    pub fn reread(&mut self) -> Option<Act> {
        let work = self.session.reread()?;
        self.perform(work)
    }

    /// A running editor is stopped first.
    pub fn discard(&mut self) -> Option<Act> {
        let work = self.session.discard()?;
        self.perform(work)
    }

    pub fn dismiss(&mut self) {
        self.session.dismiss();
    }

    pub fn plan_started(&mut self) -> Option<Act> {
        let work = self.session.plan_started()?;
        self.perform(work)
    }

    /// A plan was refused because another one runs.
    pub fn plan_refused(&mut self) -> Option<Act> {
        let work = self.session.plan_refused()?;
        self.perform(work)
    }

    /// A plan ended. Any plan may have added or removed a file of the
    /// catalog, so the page looks again once it looked at all.
    pub fn plan_finished(&mut self, succeeded: bool) -> Option<Act> {
        if self.listing.is_some() {
            self.list();
        }

        let work = self.session.plan_finished(succeeded)?;
        self.perform(work)
    }

    pub fn take(&mut self) -> Taken {
        let landed = self.jobs.as_ref().map(Jobs::landed).unwrap_or_default();
        let mut acts = Vec::new();
        for result in landed {
            let work = match result {
                Landed::Listed(listing) => {
                    self.listing = Some(listing);
                    self.listing_asked = false;
                    None
                }
                Landed::Fetched(result) => self.session.fetched(result),
                Landed::Compared(result) => self.session.compared(result),
                Landed::Viewed(number, result) => {
                    self.viewing.read(number, result);
                    None
                }
            };

            acts.extend(work.and_then(|work| self.perform(work)));
        }

        let phase = self.session.phase();
        let review = phase != std::mem::replace(&mut self.seen, phase)
            && matches!(
                phase,
                Phase::Review | Phase::Saved | Phase::Unchanged | Phase::Failed
            );

        Taken { acts, review }
    }

    fn target(&self, key: &str) -> Option<Target> {
        let catalog = self.catalog.as_ref().ok()?;
        target_of(catalog, self.listing.as_ref()?, key)
    }

    fn list(&mut self) {
        if let (Some(jobs), Ok(catalog)) = (&self.jobs, &self.catalog) {
            jobs.list(catalog);
            self.listing_asked = true;
        }
    }

    /// Carries out what a step of the edit asks for: work on a thread
    /// here, or what the app does.
    fn perform(&self, work: Work) -> Option<Act> {
        let jobs = self.jobs.as_ref();
        match work {
            Work::Fetch(path) => jobs.map(|jobs| jobs.fetch(path)).and(None),
            Work::RunEditor(editor, copy) => Some(Act::Run(edit_plan(editor, copy))),
            Work::Compare(copy, original) => {
                jobs.map(|jobs| jobs.compare(copy, original)).and(None)
            }
            Work::Reread(path, edited) => jobs.map(|jobs| jobs.reread(path, edited)).and(None),
            Work::Write(write) => Some(Act::Run(write_plan(write))),
            Work::RemoveCopy(copy) => jobs.map(|jobs| jobs.remove(copy)).and(None),
            Work::Cancel => Some(Act::Cancel),
        }
    }
}

#[cfg(any(test, feature = "testing"))]
impl Configs {
    /// The state a page test shows, without reads.
    pub fn showing(catalog: ConfigFileCatalog, listing: Listing) -> Self {
        Self {
            catalog: Ok(catalog),
            listing: Some(listing),
            ..Self::default()
        }
    }

    #[must_use = "returns the new value, the old one stays as it was"]
    pub fn with(mut self, session: Session, viewing: Viewing) -> Self {
        self.session = session;
        self.viewing = viewing;
        self
    }
}
