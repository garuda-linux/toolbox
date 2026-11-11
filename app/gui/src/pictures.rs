//! About 1,700 apps would make a lookup of every one at start slow, so only
//! the rows on screen ask, a background task answers, and each answer is
//! kept until the icon theme changes.

mod budget;
#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use crossbeam_channel::{Receiver, Sender};
use egui::Context;
use toolbox_core::features::discover::icon_name;
use toolbox_plan::PackageName;
use toolbox_system::{AppIconSource, AppStreamIcons};

use self::budget::Budget;
use crate::desktop::icon_files::{IconLookup, IconNames, Indexed, SystemIcons};

/// The icon theme spec's own fallback, which has app pictures, used
/// while the desktop names no theme.
pub const FALLBACK_THEME: &str = "hicolor";

/// How many pictures stay decoded: a few screens of rows, so scrolling
/// back is quick, without keeping every app's picture.
const KEPT: usize = 120;

pub trait PictureFinder: Send + 'static {
    fn find(&mut self, theme: &str, package: &PackageName) -> Option<PathBuf>;
}

/// The icon theme under the package's plain name first (`brave-bin`
/// shows `brave`), then the `AppStream` icon cache, read at its first
/// use.
pub struct Layered<L, S> {
    theme: L,
    read: Option<Box<dyn FnOnce() -> S + Send>>,
    cache: Option<S>,
}

pub type SystemPictures = Layered<Indexed<SystemIcons>, AppStreamIcons>;

#[derive(Debug, Default)]
struct Known {
    theme: Option<String>,
    files: HashMap<PackageName, Lookup>,
}

#[derive(Debug)]
enum Lookup {
    Asked,
    Done(Option<PathBuf>),
}

struct Answer {
    theme: String,
    package: PackageName,
    file: Option<PathBuf>,
}

/// The UI's end. Only the UI thread uses it, so the answers it keeps sit
/// in a `RefCell` and the pages ask through a shared reference.
pub struct Pictures {
    asks: Sender<(String, PackageName)>,
    answers: Receiver<Answer>,
    known: RefCell<Known>,
    budget: RefCell<Budget>,
}

impl<L: IconLookup, S: AppIconSource> Layered<L, S> {
    pub fn new(theme: L, read: impl FnOnce() -> S + Send + 'static) -> Self {
        Self {
            theme,
            read: Some(Box::new(read)),
            cache: None,
        }
    }
}

impl SystemPictures {
    pub fn system() -> Self {
        Self::new(
            Indexed::new(SystemIcons::APPS, IconNames::system),
            AppStreamIcons::system,
        )
    }
}

impl<L, S> PictureFinder for Layered<L, S>
where
    L: IconLookup + Send + 'static,
    S: AppIconSource + 'static,
{
    fn find(&mut self, theme: &str, package: &PackageName) -> Option<PathBuf> {
        // Trace: one per row, for the profiler alone.
        let _find = tracing::trace_span!("picture").entered();
        if let Some(file) = self.theme.find(theme, icon_name(package)) {
            return Some(file);
        }

        if let Some(read) = self.read.take() {
            let _read = tracing::debug_span!("read", what = "the AppStream icon cache").entered();
            self.cache = Some(read());
        }

        self.cache.as_ref()?.icon_file(package.as_str())
    }
}

impl Pictures {
    /// Starts the background thread. It ends with the returned value, so
    /// it never holds up the shutdown of a runtime as a blocking task would.
    pub fn spawn(ctx: Context, mut finder: impl PictureFinder) -> Self {
        let (asks, asked) = crossbeam_channel::unbounded::<(String, PackageName)>();
        let (answer, answers) = crossbeam_channel::unbounded();

        // A thread that cannot start behaves like one that ended at once:
        // every row keeps its glyph.
        let started = std::thread::Builder::new()
            .name("pictures".to_owned())
            .spawn(move || {
                for (theme, package) in asked {
                    let file = finder.find(&theme, &package);
                    let sent = answer.send(Answer {
                        theme,
                        package,
                        file,
                    });

                    if sent.is_err() {
                        return;
                    }

                    ctx.request_repaint();
                }
            });

        if let Err(error) = started {
            tracing::warn!("the app pictures are unavailable: {error}");
        }

        Self {
            asks,
            answers,
            known: RefCell::default(),
            budget: RefCell::new(Budget::new(KEPT)),
        }
    }

    /// The icon theme the pictures come from. Another one forgets every
    /// answer.
    pub fn set_theme(&self, theme: &str) {
        let mut known = self.known.borrow_mut();
        if known.theme.as_deref() != Some(theme) {
            known.theme = Some(theme.to_owned());
            known.files.clear();
        }
    }

    pub fn file(&self, package: &PackageName) -> Option<PathBuf> {
        let mut borrowed = self.known.borrow_mut();
        let known = &mut *borrowed;
        let theme = known.theme.clone()?;

        for answer in self.answers.try_iter() {
            if answer.theme == theme {
                known
                    .files
                    .insert(answer.package, Lookup::Done(answer.file));
            }
        }

        match known.files.get(package) {
            Some(Lookup::Done(file)) => file.clone(),
            Some(Lookup::Asked) => None,
            None => {
                // An error means the task ended. The row shows no picture.
                self.asks.send((theme, package.clone())).ok();
                known.files.insert(package.clone(), Lookup::Asked);
                None
            }
        }
    }

    /// Notes that a row drew the picture at `uri`. The one drawn longest
    /// ago leaves egui's memory once more than [`KEPT`] were drawn.
    pub fn drawn(&self, ctx: &Context, uri: &str) {
        let leaving = self.budget.borrow_mut().drawn(uri);
        if let Some(leaving) = leaving {
            ctx.forget_image(&leaving);
        }
    }

    #[cfg(test)]
    fn waiting(&self, package: &PackageName) -> bool {
        matches!(self.known.borrow().files.get(package), Some(Lookup::Asked))
    }
}

impl std::fmt::Debug for Pictures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pictures").finish_non_exhaustive()
    }
}
