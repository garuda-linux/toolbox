//! The extension point for queued changes.
//!
//! A feature that lets the user change something adds one [`Domain`]: what a
//! change targets, what the user wishes for it, the part of the system snapshot
//! it compares with, where it runs in a plan, the actions that carry it out and
//! how the Review page shows it. Nothing else in the core changes for it.

use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use toolbox_plan::PlannedAction;
/// The user's wish for something that can be on or off, shared with the
/// probes that read the current state.
pub use toolbox_state::Toggle;

use crate::Snapshot;
use crate::outcome::Outcome;

/// The wish of a domain whose changes are work to do, not a state to
/// reach: being queued says it all, so no "off" can be filed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Queued;

/// The item of a domain with one setting for the whole system, such as
/// the DNS provider: the wish is all there is to a change.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Single;

/// Where a domain's actions go in a plan. Lower runs first. The named
/// stages leave gaps, so a new domain can slot in between without moving
/// the others.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Stage(u16);

impl Stage {
    /// Switching off what a later stage takes away: a unit is disabled
    /// while the package that ships it is still installed.
    pub const TEARDOWN: Self = Self(50);
    pub const REPOSITORIES: Self = Self(100);
    pub const PACKAGES: Self = Self(200);
    /// A build needs the kernel headers the package stage may install.
    pub const DKMS_BUILDS: Self = Self(350);
    pub const ACCOUNTS: Self = Self(400);
    pub const FILES: Self = Self(500);
    pub const SERVICES: Self = Self(600);
    pub const LOCALES: Self = Self(700);
    pub const BOOT: Self = Self(800);
    pub const USER_SESSION: Self = Self(900);

    pub const fn new(rank: u16) -> Self {
        Self(rank)
    }

    pub const fn rank(self) -> u16 {
        self.0
    }
}

/// One action and where it runs in the plan. A domain's actions need not
/// share a stage: switching off goes to [`Stage::TEARDOWN`], switching on
/// to the domain's own stage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Staged {
    pub stage: Stage,
    pub action: PlannedAction,
}

impl Staged {
    #[must_use = "a staged action does nothing until it is planned"]
    pub const fn at(stage: Stage, action: PlannedAction) -> Self {
        Self { stage, action }
    }
}

/// The UI owns the sentences, so the core holds no translated text and the
/// page takes no key apart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Summary {
    pub code: &'static str,
    /// What the sentence names, in the order of its placeables.
    pub values: Vec<String>,
}

impl Summary {
    pub const fn plain(code: &'static str) -> Self {
        Self {
            code,
            values: Vec::new(),
        }
    }

    pub fn of(code: &'static str, value: impl Into<String>) -> Self {
        Self {
            code,
            values: vec![value.into()],
        }
    }

    pub fn pair(code: &'static str, first: impl Into<String>, second: impl Into<String>) -> Self {
        Self {
            code,
            values: vec![first.into(), second.into()],
        }
    }

    pub fn toggle(
        wish: Toggle,
        on: &'static str,
        off: &'static str,
        value: impl Into<String>,
    ) -> Self {
        Self::of(if wish.is_on() { on } else { off }, value)
    }
}

/// One queued change, as the Review page lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeView {
    pub domain: &'static str,
    pub key: String,
    pub summary: Summary,
    pub outcome: Outcome,
}

impl ChangeView {
    /// Whether applying it takes away something the user has: a package,
    /// a group or an account. Such a queue is shown before it is applied.
    pub fn removes(&self) -> bool {
        matches!(
            self.summary.code,
            "package-remove" | "group-remove" | "user-remove-keep-home" | "user-remove-with-home"
        )
    }
}

/// Implementations are plain types without state: everything they need comes
/// in through the arguments.
pub trait Domain: 'static {
    /// What a change targets, for example a package name.
    type Item: Clone + Debug + Ord + Serialize + DeserializeOwned + Send + Sync + 'static;
    type Wish: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Send + Sync + 'static;
    /// The part of the [`crate::Snapshot`] this domain compares with.
    type State: Send + Sync + 'static;

    /// Stable name, used by the UI and in logs.
    const ID: &'static str;
    /// Where the domain's changes are listed and, unless `plan` says
    /// otherwise for an action, where they run.
    const STAGE: Stage;
    /// One-off work (clean the cache, upgrade the system) rather than a
    /// state of the system. No probe can see that it ran, so
    /// [`crate::ChangeSet::drop_done_one_offs`] removes it once its steps
    /// succeeded.
    const ONE_OFF: bool = false;

    /// This domain's part of `snapshot`, `None` while no probe delivered it
    /// (every wish then stays queued). A domain without a probe returns its
    /// state directly.
    fn state(snapshot: &Snapshot) -> Option<&Self::State> {
        snapshot.part::<Self::State>()
    }

    /// Whether `wish` belongs to `item`. A wish that names its own target
    /// as well (a GRUB setting names its variable) must name this one, or
    /// a saved file could file it under another target.
    fn fits(_item: &Self::Item, _wish: &Self::Wish) -> bool {
        true
    }

    fn is_satisfied(item: &Self::Item, wish: &Self::Wish, state: &Self::State) -> bool;

    /// The actions for all queued changes of this domain, each with its
    /// stage. Batching (one pacman call for many packages) happens here.
    ///
    /// Every action belongs to all the domain's changes. One that does not
    /// end well keeps each with [`Outcome::Failed`], even when
    /// [`Domain::is_satisfied`] sees the wish: a `locale.gen` edit can
    /// succeed while `locale-gen` fails, and the change must be tried again.
    fn plan(changes: &[(&Self::Item, &Self::Wish)]) -> Vec<Staged>;

    /// Never taken apart again: what a sentence names comes from
    /// [`Domain::summary`].
    fn key(item: &Self::Item) -> String;

    fn summary(item: &Self::Item, wish: &Self::Wish) -> Summary;
}
