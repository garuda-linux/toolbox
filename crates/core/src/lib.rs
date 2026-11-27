//! The app's logic without I/O: queued changes, the system snapshot they
//! are compared with, and the planner that orders them. Probes and runners
//! pass their results in.
//!
//! A new kind of change is a new [`Domain`] (see `domains/`). To survive a
//! restart, it also needs its line in [`domains::all`].

#[macro_use]
mod macros;

mod changes;
pub mod domain;
pub mod domains;
pub mod features;
mod outcome;
mod planner;
mod saved;
mod summary_code;
#[cfg(test)]
mod test_support;
#[cfg(any(test, feature = "testing"))]
pub mod testing;

pub use changes::ChangeSet;
pub use domain::{ChangeView, Domain, Queued, Single, Stage, Staged, Summary, Toggle};
pub use outcome::{Outcome, Owners};
pub use planner::{plan, planned};
pub use saved::{Domains, RegistryError, RestoreProblem, SAVED_VERSION, SavedChange, SavedChanges};
pub use summary_code::SummaryCode;
pub use toolbox_state::Snapshot;
