//! What the window and the command line share without egui: carrying out
//! a plan, the saved queue, the catalogs, diagnostics, the texts, the page
//! names and the wake a service calls after each value. It links no
//! toolkit, so the command line can build on it without one.

pub mod apply;
pub mod catalogs;
pub mod configs;
pub mod desktop;
pub mod diagnostics;
pub mod i18n;
pub mod instance;
pub mod log_folder;
pub mod page;
pub mod programs;
pub mod queue;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
pub mod units;
pub mod waking;
