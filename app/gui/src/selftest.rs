//! The scripted self-test inside the real window.
//!
//! `scripts/selftest.sh` starts the app on a headless Wayland
//! compositor with the dry-run helper, and a suite clicks through the
//! pages, saving screenshots on the way. Only built with the `selftest`
//! feature.

mod about;
mod boot;
mod bootrepair;
mod components;
mod configs;
mod diagnostics;
mod discover;
mod gaming;
mod guide;
mod home;
mod kernels;
mod locales;
mod maintenance;
mod network;
mod packages;
mod runner;
mod services;
mod settings;
mod setup;
mod step;
#[cfg(test)]
mod tests;
mod tweaks;
mod users;

use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

use egui::Context;
pub use runner::{RESULT_FILE, Runner};
pub use step::Step;

/// The folder for the screenshots and the verdict. Unset, there is no
/// self-test.
const DIR_VARIABLE: &str = "GARUDA_TOOLBOX_SELFTEST_DIR";
/// The suite to run, `all` when unset.
const SUITE_VARIABLE: &str = "GARUDA_TOOLBOX_SELFTEST_SUITE";
/// How long one step may wait. A cold first read of the system is slow.
const PATIENCE: Duration = Duration::from_secs(60);

const SUITES: [(&str, Suite); 21] = [
    ("home", home::steps),
    ("setup", setup::steps),
    ("components", components::steps),
    ("gaming", gaming::steps),
    ("discover", discover::steps),
    ("packages", packages::steps),
    ("packages-aur", packages::aur_steps),
    ("services", services::steps),
    ("tweaks", tweaks::steps),
    ("kernels", kernels::steps),
    ("locales", locales::steps),
    ("network", network::steps),
    ("users", users::steps),
    ("configs", configs::steps),
    ("bootrepair", bootrepair::steps),
    ("boot", boot::steps),
    ("maintenance", maintenance::steps),
    ("diagnostics", diagnostics::steps),
    ("settings", settings::steps),
    ("guide", guide::steps),
    ("about", about::steps),
];

/// Suites that need what only the Arch runner has, such as pacman's
/// databases. `all` leaves them out, CI runs each by name.
const ARCH_SUITES: [(&str, Suite); 1] = [("packages-arch", packages::arch_steps)];

type Suite = fn() -> Vec<Step>;

pub fn requested(lookup: impl Fn(&str) -> Option<OsString>) -> Option<(PathBuf, String)> {
    let dir = lookup(DIR_VARIABLE).filter(|dir| !dir.is_empty())?;
    let suite = lookup(SUITE_VARIABLE)
        .and_then(|suite| suite.into_string().ok())
        .filter(|suite| !suite.is_empty())
        .unwrap_or_else(|| "all".to_owned());

    Some((PathBuf::from(dir), suite))
}

pub fn steps(suite: &str) -> Option<Vec<Step>> {
    if suite == "all" {
        return Some(SUITES.iter().flat_map(|(_, steps)| steps()).collect());
    }

    SUITES
        .iter()
        .chain(ARCH_SUITES.iter())
        .find(|(name, _)| *name == suite)
        .map(|(_, steps)| steps())
}

/// # Errors
///
/// The name of a suite that does not exist.
pub fn install(ctx: &Context, dir: PathBuf, suite: &str) -> Result<(), String> {
    let steps = steps(suite).ok_or_else(|| format!("there is no self-test suite {suite}"))?;

    ctx.enable_accesskit();
    ctx.add_plugin(Runner::new(steps, dir, PATIENCE));
    Ok(())
}
