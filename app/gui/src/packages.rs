//! What the Packages page keeps between frames: the index of every
//! package, and the details and history of the package it shows.
//!
//! Reads run when the page opens, on refresh and after every plan. libalpm
//! and the log block, so the reads run on the runtime and wake the frame
//! with their result. The AUR's copy is read only once its chip shows, and
//! again only on Read again: a plan changes nothing in it.

mod edits;
mod orders;
#[cfg(any(test, feature = "selftest", feature = "testing"))]
mod sample;
#[cfg(test)]
mod tests;

use std::fmt;
use std::sync::Arc;

use tokio::runtime::Handle;
use toolbox_aur::{AurError, AurSource};
use toolbox_core::features::packages::PackageDetails;
use toolbox_core::features::packages::aur::{AurCatalog, AurNames};
use toolbox_core::features::packages::history::{HistoryEntry, history};
use toolbox_core::features::packages::list::Catalog;
use toolbox_plan::PackageName;
use toolbox_services::aur_orders::OrderReads;
use toolbox_system::{AurOrderSource, PackageIndexSource, RecentLogSource, RouteSource};

pub use self::orders::Landing;
#[cfg(any(test, feature = "selftest", feature = "testing"))]
pub use self::sample::SampleAur;
use crate::lane::Lane;
pub use crate::network::Read;
use crate::refused::Refused;

pub struct PackagesSetup {
    pub runtime: Handle,
    pub source: Arc<dyn PackageIndexSource>,
    pub log: Arc<dyn RecentLogSource>,
    pub aur: Arc<dyn AurSource>,
    /// The build orders of queued AUR builds, read by paru as the user.
    pub orders: Arc<dyn AurOrderSource>,
    /// The route check the helper makes for this user, read again for
    /// every order read.
    pub route: Arc<dyn RouteSource>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Details {
    pub package: PackageName,
    /// `Known(None)` when no database has the package any more.
    pub info: Read<Option<PackageDetails>>,
    pub history: Read<Vec<HistoryEntry>>,
}

#[derive(Debug, Default)]
struct Reads {
    index: bool,
    info: bool,
    history: bool,
}

#[derive(Debug, Default)]
pub struct Packages {
    opened: bool,
    index: Read<Arc<Catalog>>,
    aur_asked: bool,
    aur: Read<Arc<AurCatalog>>,
    /// The copy's names for the queue's guard, once read.
    aur_names: Option<AurNames>,
    /// A copy landed since the app last asked.
    aur_landed: bool,
    /// A new allocation with every answer, so a page tells new details from
    /// the ones its rows were built from without comparing them.
    details: Option<Arc<Details>>,
    refusal: Option<PackagesRefusal>,
    /// `None` in tests that need no reads.
    jobs: Option<Jobs>,
    /// Which reads run. The old answers stay on screen meanwhile.
    reads: Reads,
    /// The build orders of the queue, see [`Packages::ask_orders`].
    orders: OrderReads,
    /// The read that landed, until App applies it to the queue.
    landing: Option<Landing>,
    /// The queue's revision when an Apply waits for a fresh read of the
    /// orders, see [`Packages::start_apply`].
    apply: Option<u64>,
}

pub type PackagesRefusal = Refused<toolbox_core::features::packages::queue::Refusal, PackageName>;

enum Landed {
    Index(Read<Arc<Catalog>>),
    Aur(Read<Arc<AurCatalog>>, Option<AurNames>),
    Info(PackageName, Box<Read<Option<PackageDetails>>>),
    History(PackageName, Read<Vec<HistoryEntry>>),
    Orders(Landing),
}

struct Jobs {
    setup: PackagesSetup,
    lane: Lane<Landed>,
}

impl Packages {
    pub fn new(setup: Option<PackagesSetup>, ctx: egui::Context) -> Self {
        Self {
            jobs: setup.map(|setup| Jobs {
                lane: Lane::new(setup.runtime.clone(), crate::waking::repaint(ctx)),
                setup,
            }),
            ..Self::default()
        }
    }

    pub const fn index(&self) -> &Read<Arc<Catalog>> {
        &self.index
    }

    /// Whether a read of the index or of the shown details runs.
    pub const fn reading(&self) -> bool {
        self.reads.index || self.reads.info || self.reads.history
    }

    pub const fn aur(&self) -> &Read<Arc<AurCatalog>> {
        &self.aur
    }

    /// The names of the copy, once read, for the snapshot the queue's
    /// guard checks against.
    pub const fn aur_names(&self) -> Option<&AurNames> {
        self.aur_names.as_ref()
    }

    /// Whether a copy landed since the last call, so the queue meets it.
    pub const fn aur_landed(&mut self) -> bool {
        std::mem::replace(&mut self.aur_landed, false)
    }

    /// The AUR chip shows: its first look reads the copy, which downloads
    /// the dump when the last attempt is a day old.
    pub fn open_aur(&mut self) {
        if !self.aur_asked {
            self.aur_asked = true;
            self.read_aur();
        }
    }

    pub const fn details(&self) -> Option<&Arc<Details>> {
        self.details.as_ref()
    }

    /// The page shows: its first look reads the index.
    pub fn open(&mut self) {
        if !self.opened {
            self.opened = true;
            self.read_index();
        }
    }

    /// Reads the index, the AUR's copy once asked and the shown package
    /// again.
    pub fn refresh(&mut self) {
        self.read_local();
        if self.aur_asked {
            self.read_aur();
        }
    }

    /// A plan may have installed or removed packages, but changed nothing
    /// of the AUR. Reads again once the page was opened.
    pub fn plan_finished(&mut self) {
        if self.opened {
            self.read_local();
        }
    }

    fn read_local(&mut self) {
        self.read_index();
        if let Some(package) = self.details.as_ref().map(|shown| shown.package.clone()) {
            self.read_details(&package);
        }
    }

    pub fn show(&mut self, package: &PackageName) {
        if self
            .details
            .as_ref()
            .is_some_and(|shown| shown.package == *package)
        {
            return;
        }

        self.details = Some(Arc::new(Details {
            package: package.clone(),
            info: Read::Loading,
            history: Read::Loading,
        }));
        self.read_details(package);
    }

    /// An answer for a package the view left is dropped.
    pub fn take(&mut self) {
        let landed = self.jobs.as_ref().map(|jobs| jobs.lane.landed());

        for result in landed.into_iter().flatten() {
            match result {
                Landed::Index(index) => {
                    self.index = index;
                    self.reads.index = false;
                }
                Landed::Aur(aur, names) => {
                    self.aur = aur;
                    if names.is_some() {
                        self.aur_names = names;
                        self.aur_landed = true;
                    }
                }
                Landed::Info(package, info) => {
                    self.reads.info = false;
                    self.change_shown(&package, |shown| shown.info = *info);
                }
                Landed::History(package, lines) => {
                    self.reads.history = false;
                    self.change_shown(&package, |shown| shown.history = lines);
                }
                Landed::Orders(landing) => self.landing = Some(landing),
            }
        }
    }

    fn change_shown(&mut self, package: &PackageName, change: impl FnOnce(&mut Details)) {
        if let Some(shown) = self
            .details
            .as_mut()
            .filter(|shown| shown.package == *package)
        {
            let mut changed = Details::clone(shown);
            change(&mut changed);
            *shown = Arc::new(changed);
        }
    }

    fn read_index(&mut self) {
        let Some(jobs) = &self.jobs else {
            return;
        };

        self.reads.index = true;
        let source = Arc::clone(&jobs.setup.source);
        jobs.lane.blocking(
            "index",
            move || Read::of("the package index", source.index().map(Catalog::new)),
            |read| Landed::Index(read.map_or(Read::Failed, |read| read.map(Arc::new))),
        );
    }

    fn read_aur(&self) {
        let Some(jobs) = &self.jobs else {
            return;
        };

        let source = Arc::clone(&jobs.setup.aur);
        jobs.lane.blocking(
            "aur",
            move || match source.index() {
                Ok(index) => {
                    let catalog = AurCatalog::new(index);
                    let names = catalog.names();
                    Landed::Aur(Read::Known(Arc::new(catalog)), Some(names))
                }
                Err(error) => {
                    tracing::warn!("{error}");
                    let read = match error {
                        AurError::Waiting { next: _ } => Read::Unavailable,
                        AurError::Download(_) | AurError::Storage { path: _, error: _ } => {
                            Read::Failed
                        }
                    };

                    Landed::Aur(read, None)
                }
            },
            |landed| landed.unwrap_or(Landed::Aur(Read::Failed, None)),
        );
    }

    fn read_details(&mut self, package: &PackageName) {
        let Some(jobs) = &self.jobs else {
            return;
        };

        self.reads.info = true;
        self.reads.history = true;
        let source = Arc::clone(&jobs.setup.source);
        let wanted = package.clone();
        let answer = package.clone();
        jobs.lane.blocking(
            "info",
            move || Read::of("the package's details", source.details(&wanted)),
            move |read| Landed::Info(answer, Box::new(read.unwrap_or(Read::Failed))),
        );

        let log = Arc::clone(&jobs.setup.log);
        let asked = package.clone();
        let landing = package.clone();
        jobs.lane.blocking(
            "history",
            move || {
                let lines = log.recent().map(|text| history(&text, &asked));
                Read::of("pacman's log", lines)
            },
            move |read| Landed::History(landing, read.unwrap_or(Read::Failed)),
        );
    }
}

impl<T> Read<T> {
    pub fn map<U>(self, change: impl FnOnce(T) -> U) -> Read<U> {
        match self {
            Self::Known(value) => Read::Known(change(value)),
            Self::Loading => Read::Loading,
            Self::Unavailable => Read::Unavailable,
            Self::Failed => Read::Failed,
        }
    }
}

#[cfg(any(test, feature = "testing"))]
impl Packages {
    pub fn showing(index: Read<Arc<Catalog>>, details: Option<Details>) -> Self {
        Self {
            opened: true,
            index,
            aur_asked: true,
            aur: Read::Loading,
            aur_names: None,
            aur_landed: false,
            details: details.map(Arc::new),
            refusal: None,
            jobs: None,
            reads: Reads::default(),
            orders: OrderReads::default(),
            landing: None,
            apply: None,
        }
    }

    #[must_use = "returns the new value, the old one stays as it was"]
    pub fn with_aur(self, aur: Read<Arc<AurCatalog>>) -> Self {
        Self { aur, ..self }
    }
}

impl fmt::Debug for Jobs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Jobs").finish_non_exhaustive()
    }
}

impl fmt::Debug for PackagesSetup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PackagesSetup").finish_non_exhaustive()
    }
}
