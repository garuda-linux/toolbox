//! The app's logic without I/O: queued changes, the system snapshot they
//! are compared with, and the planner that orders them. Probes and runners
//! live elsewhere and pass their results in.
//!
//! New kinds of changes are new [`Domain`]s (see `domains/` for examples).
//! The change set, the snapshot and the planner work with any of them. To
//! survive a restart, a domain also needs its line in [`domains::all`].

#[macro_use]
mod macros;

mod changes;
pub mod domain;
pub mod domains;
pub mod features;
mod outcome;
mod planner;
mod saved;
#[cfg(test)]
mod test_support;
#[cfg(any(test, feature = "testing"))]
pub mod testing;

pub use changes::ChangeSet;
pub use domain::{ChangeView, Domain, Queued, Single, Stage, Staged, Summary, Toggle};
pub use outcome::{Outcome, Owners};
pub use planner::{plan, planned};
pub use saved::{
    Domains, DuplicateDomain, RestoreProblem, SAVED_VERSION, SavedChange, SavedChanges,
};
pub use toolbox_state::Snapshot;
