//! A library beside `main.rs`, so the tests build the same [`App`] the binary runs.

mod app;
pub mod appmenu;
pub mod artwork;
pub mod boot;
pub mod bootrepair;
pub mod catalogs;
pub mod command;
pub mod configs;
pub mod console;
pub mod dates;
#[cfg(debug_assertions)]
pub mod debug_overlay;
pub mod desktop;
#[cfg(test)]
mod folding;
pub mod home;
pub mod i18n;
pub mod instance;
pub mod kernels;
pub mod lane;
pub mod launch;
pub mod links;
pub mod locales;
pub mod maintenance;
pub mod missing;
pub mod network;
#[cfg(any(test, feature = "selftest", feature = "testing"))]
pub mod news;
pub mod packages;
pub mod page_service;
pub mod page_ui;
pub mod pages;
pub mod panics;
pub mod pictures;
pub mod power;
pub mod probe_texts;
pub mod profiling;
pub mod reader;
pub mod refused;
pub mod schemes;
#[cfg(feature = "selftest")]
pub mod selftest;
pub mod settings;
pub mod share;
mod shell;
pub mod snapshot;
pub mod spawn;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
pub mod toggle_pages;
pub mod units;
pub mod users;
pub mod waking;
pub mod zoom;

pub use app::{App, FIXED_VERSION, PreferencesSetup, Setup};
pub use toolbox_services::{apply, diagnostics, log_folder, page, programs, queue};
