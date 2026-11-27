//! The samples a self-test build's dry run answers with, so the self-test
//! never reaches the network nor shows the disks it runs on. A release
//! build has none and reads the real sources, which change nothing.

use std::path::PathBuf;
use std::sync::Arc;

use toolbox_aur::AurSource;
use toolbox_news::NewsSource;
use toolbox_system::{AurOrderSource, MergeSource, RepairSource, RouteSource};

#[cfg(feature = "selftest")]
#[expect(
    clippy::unnecessary_wraps,
    reason = "a release build answers None through the same signature"
)]
pub fn repair() -> Option<Arc<dyn RepairSource>> {
    Some(Arc::new(toolbox_system::DryRunRepair))
}

#[cfg(feature = "selftest")]
#[expect(
    clippy::unnecessary_wraps,
    reason = "a release build answers None through the same signature"
)]
pub fn aur() -> Option<Arc<dyn AurSource>> {
    Some(Arc::new(toolbox_gui::packages::SampleAur))
}

#[cfg(feature = "selftest")]
#[expect(
    clippy::unnecessary_wraps,
    reason = "a release build answers None through the same signature"
)]
pub fn aur_orders() -> Option<Arc<dyn AurOrderSource>> {
    Some(Arc::new(toolbox_system::SampleAurOrders))
}

#[cfg(feature = "selftest")]
#[expect(
    clippy::unnecessary_wraps,
    reason = "a release build answers None through the same signature"
)]
pub fn route() -> Option<Arc<dyn RouteSource>> {
    Some(Arc::new(toolbox_system::SampleRoute))
}

#[cfg(not(feature = "selftest"))]
pub const fn aur_orders() -> Option<Arc<dyn AurOrderSource>> {
    None
}

#[cfg(not(feature = "selftest"))]
pub const fn route() -> Option<Arc<dyn RouteSource>> {
    None
}

#[cfg(feature = "selftest")]
#[expect(
    clippy::unnecessary_wraps,
    reason = "a release build answers None through the same signature"
)]
pub fn news() -> Option<Arc<dyn NewsSource>> {
    Some(Arc::new(toolbox_gui::news::SampleNews))
}

#[cfg(not(feature = "selftest"))]
pub const fn repair() -> Option<Arc<dyn RepairSource>> {
    None
}

#[cfg(not(feature = "selftest"))]
pub const fn aur() -> Option<Arc<dyn AurSource>> {
    None
}

#[cfg(not(feature = "selftest"))]
pub const fn news() -> Option<Arc<dyn NewsSource>> {
    None
}

#[cfg(feature = "selftest")]
#[expect(
    clippy::unnecessary_wraps,
    reason = "a release build answers None through the same signature"
)]
pub fn merges(root: PathBuf) -> Option<Arc<dyn MergeSource>> {
    Some(Arc::new(toolbox_system::FoundCopies::new(root)))
}

#[cfg(not(feature = "selftest"))]
pub fn merges(_root: PathBuf) -> Option<Arc<dyn MergeSource>> {
    None
}
