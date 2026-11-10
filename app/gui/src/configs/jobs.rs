//! The blocking parts of the Config Files page, each on a blocking thread
//! of the runtime: listing the files, reading one, comparing the copy the
//! editor left. Every result wakes the frame.

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::runtime::Handle;
use toolbox_catalog::{ConfigFileCatalog, LoadProblem};
use toolbox_core::features::configs::session::{Changes, Compared, EditedFile, Fetched, Problem};
use toolbox_core::features::configs::viewing::Viewed;
use toolbox_core::features::configs::{Diff, Listing, Target, choose_editor};
use toolbox_plan::{ConfigFilePath, ConfigText, EditCopy, Editor, FetchRefusal, InvalidConfigText};
use toolbox_services::configs::specs;
use toolbox_system::{ConfigFileSource, EditFolder, Editing, ReadProblem};

use super::turns::{Turn, Turns};
use crate::lane::Lane;

pub struct ConfigsSetup {
    pub runtime: Handle,
    pub source: Arc<dyn ConfigFileSource>,
    /// A broken catalog only empties the page, with the reason.
    pub catalog: Result<ConfigFileCatalog, LoadProblem>,
    pub copies: EditFolder,
    pub editing: Arc<dyn Editing>,
    /// `$VISUAL` and `$EDITOR`, which only pick from the known editors.
    pub visual: Option<String>,
    pub editor: Option<String>,
    /// A dry run opens no app for a file of the user.
    pub dry_run: bool,
}

pub enum Landed {
    Listed(Listing),
    Fetched(Result<Fetched, Problem>),
    Compared(Result<Compared, Problem>),
    Viewed(u64, Result<Viewed, Problem>),
}

#[derive(Clone)]
struct Tools {
    source: Arc<dyn ConfigFileSource>,
    copies: EditFolder,
    editing: Arc<dyn Editing>,
    visual: Option<String>,
    editor: Option<String>,
    /// Set when the page's jobs end with the app, so a password dialog no
    /// longer holds a read.
    cancel: Arc<AtomicBool>,
}

pub struct Jobs {
    runtime: Handle,
    dry_run: bool,
    tools: Tools,
    ctx: egui::Context,
    lane: Lane<Landed>,
    /// Every job on the folder of copies runs in the order it was asked.
    folder: Turns,
}

impl fmt::Debug for ConfigsSetup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigsSetup").finish_non_exhaustive()
    }
}

impl fmt::Debug for Jobs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Jobs").finish_non_exhaustive()
    }
}

impl Drop for Jobs {
    fn drop(&mut self) {
        self.tools.cancel.store(true, Ordering::Relaxed);
    }
}

impl Jobs {
    pub fn new(
        setup: ConfigsSetup,
        ctx: egui::Context,
    ) -> (Self, Result<ConfigFileCatalog, LoadProblem>) {
        let jobs = Self {
            lane: Lane::new(setup.runtime.clone(), crate::waking::repaint(ctx.clone())),
            runtime: setup.runtime,
            dry_run: setup.dry_run,
            tools: Tools {
                source: setup.source,
                copies: setup.copies,
                editing: setup.editing,
                visual: setup.visual,
                editor: setup.editor,
                cancel: Arc::default(),
            },
            ctx,
            folder: Turns::default(),
        };

        // A copy left by a crash may hold what only root can read.
        jobs.quietly(Tools::clear);
        (jobs, setup.catalog)
    }

    pub fn landed(&self) -> Vec<Landed> {
        self.lane.landed()
    }

    pub fn open_home(&self, target: &Target) -> bool {
        let Target::Home(path) = target else {
            return false;
        };

        let Some(file) = self.tools.source.home_file(path) else {
            return false;
        };

        if self.dry_run {
            tracing::info!("dry run, not opening {}", file.display());
        } else {
            crate::links::open_file(&self.ctx, &file);
        }

        true
    }

    pub fn list(&self, catalog: &ConfigFileCatalog) {
        let specs = specs(catalog);
        self.blocking(
            "list",
            move |tools| Landed::Listed(tools.source.listing(&specs)),
            || Landed::Listed(Listing::default()),
        );
    }

    pub fn fetch(&self, file: EditedFile) {
        let name = match &file {
            EditedFile::Root(path) => Target::Root(path.clone()),
            EditedFile::Home(path) => Target::Home(path.clone()),
        }
        .file_name()
        .to_owned();

        let turn = self.folder.take();
        self.blocking(
            "fetch",
            move |tools| Landed::Fetched(tools.fetch(&file, &name, turn)),
            || Landed::Fetched(Err(stopped())),
        );
    }

    pub fn view(&self, number: u64, path: ConfigFilePath) {
        self.blocking(
            "view",
            move |tools| Landed::Viewed(number, tools.read(&path)),
            move || Landed::Viewed(number, Err(stopped())),
        );
    }

    pub fn compare(&self, copy: EditCopy, original: ConfigText) {
        let mut turn = self.folder.take();
        self.blocking(
            "compared",
            move |tools| {
                turn.wait();
                Landed::Compared(tools.compare(&copy, original))
            },
            || Landed::Compared(Err(stopped())),
        );
    }

    pub fn reread(&self, file: EditedFile, edited: ConfigText) {
        self.blocking(
            "compared",
            move |tools| {
                Landed::Compared(
                    tools
                        .read_edited(&file)
                        .map(|(_, content)| changes(content, edited)),
                )
            },
            || Landed::Compared(Err(stopped())),
        );
    }

    pub fn remove(&self, copy: EditCopy) {
        self.quietly(move |tools| {
            if let Err(error) = tools.copies.remove(&copy) {
                tracing::warn!("cannot remove the copy {copy}: {error}");
            }
        });
    }

    fn blocking<W, S>(&self, kind: &'static str, work: W, stopped: S)
    where
        W: FnOnce(&Tools) -> Landed + Send + 'static,
        S: FnOnce() -> Landed + Send + 'static,
    {
        let tools = self.tools.clone();
        self.lane.blocking(
            kind,
            move || work(&tools),
            |landed| landed.unwrap_or_else(|_panicked| stopped()),
        );
    }

    fn quietly<W>(&self, work: W)
    where
        W: FnOnce(&Tools) + Send + 'static,
    {
        let tools = self.tools.clone();
        let mut turn = self.folder.take();
        drop(self.runtime.spawn_blocking(move || {
            turn.wait();
            work(&tools);
        }));
    }
}

impl Tools {
    /// Picks the editor first: without one there is no point in asking
    /// for a password.
    ///
    /// The read happens before the turn, since it may wait on a password
    /// dialog.
    fn fetch(&self, file: &EditedFile, name: &str, mut turn: Turn) -> Result<Fetched, Problem> {
        let editor = self.editor()?;
        let (target, original) = self.read_edited(file)?;
        turn.wait();
        let copy = self
            .copies
            .create(name, &original)
            .map_err(|error| Problem::new("copy-failed", error.to_string()))?;

        tracing::info!(
            "editing a copy of {target} (asked for as {file}) with {}",
            editor.program().name()
        );

        Ok(Fetched {
            editor,
            copy,
            original,
            target,
        })
    }

    /// The text of `file` and the file the write goes to. A root-owned
    /// file may lead elsewhere through its links, the helper checks it
    /// again. A file of the user is read as the user and keeps its path.
    fn read_edited(&self, file: &EditedFile) -> Result<(EditedFile, ConfigText), Problem> {
        match file {
            EditedFile::Root(path) => self
                .read(path)
                .map(|read| (EditedFile::Root(read.path), read.content)),
            EditedFile::Home(path) => {
                let real = self
                    .source
                    .home_file(path)
                    .ok_or_else(|| Problem::new("read-failed", "there is no home folder"))?;

                read_home(&real).map(|content| (file.clone(), content))
            }
        }
    }

    fn compare(&self, copy: &EditCopy, original: ConfigText) -> Result<Compared, Problem> {
        self.editing
            .after_edit(&self.copies, copy)
            .map_err(|error| Problem::new("copy-failed", error.to_string()))?;

        self.edited(copy).map(|edited| changes(original, edited))
    }

    fn clear(&self) {
        if let Err(error) = self.copies.clear() {
            tracing::warn!("cannot empty {}: {error}", self.copies.path().display());
        }
    }

    fn editor(&self) -> Result<Editor, Problem> {
        let chosen = choose_editor(self.visual.as_deref(), self.editor.as_deref(), |editor| {
            self.source.installed(editor)
        });

        self.editing
            .editor(chosen)
            .ok_or_else(|| Problem::new("no-editor", ""))
    }

    fn read(&self, path: &ConfigFilePath) -> Result<Viewed, Problem> {
        self.source
            .read(path, &self.cancel)
            .map(|read| Viewed {
                path: read.path,
                content: read.content,
            })
            .map_err(problem_of)
    }

    fn edited(&self, copy: &EditCopy) -> Result<ConfigText, Problem> {
        match self.copies.read(copy) {
            Ok(Ok(text)) => Ok(text),
            Ok(Err(InvalidConfigText::NotText)) => Err(refused(FetchRefusal::NotText)),
            Ok(Err(InvalidConfigText::TooLarge { max: _ })) => Err(refused(FetchRefusal::TooLarge)),
            Err(error) => Err(Problem::new("copy-failed", error.to_string())),
        }
    }
}

/// A step that panicked, in the words of a failed read.
fn stopped() -> Problem {
    Problem::new("failed", "the worker stopped")
}

fn problem_of(problem: ReadProblem) -> Problem {
    match problem {
        ReadProblem::Refused(reason) => refused(reason),
        ReadProblem::NotRun(refusal) => Problem::new(refusal.code(), ""),
        ReadProblem::Failed(reason) => Problem::new("read-failed", reason),
        ReadProblem::Cancelled => Problem::new("read-failed", "cancelled"),
    }
}

/// Links are followed, as any program of the user would.
fn read_home(file: &std::path::Path) -> Result<ConfigText, Problem> {
    let bytes = std::fs::read(file).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            refused(FetchRefusal::Missing)
        } else {
            Problem::new("read-failed", error.to_string())
        }
    })?;

    let text = String::from_utf8(bytes).map_err(|error| {
        tracing::debug!("not UTF-8: {error}");
        refused(FetchRefusal::NotText)
    })?;

    ConfigText::new(text).map_err(|invalid| match invalid {
        InvalidConfigText::TooLarge { max: _ } => refused(FetchRefusal::TooLarge),
        InvalidConfigText::NotText => refused(FetchRefusal::NotText),
    })
}

fn refused(reason: FetchRefusal) -> Problem {
    // The page's own sentence names every refusal, see `RESTATED`.
    Problem::new(reason.as_str(), "")
}

fn changes(original: ConfigText, edited: ConfigText) -> Compared {
    if original == edited {
        return Compared::Unchanged;
    }

    let diff = Diff::new(original.as_str(), edited.as_str());
    let (plain, colored) = (diff.plain(), diff.colored());
    let (added, removed) = (diff.added(), diff.removed());
    Compared::Changed(Changes {
        original,
        edited,
        plain,
        colored,
        added,
        removed,
    })
}
