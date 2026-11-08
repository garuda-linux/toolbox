//! A library beside `main.rs`, so the tests build the same [`App`] the binary runs.

mod app;
pub mod boot;
pub mod bootrepair;
pub mod catalogs;
mod clipboard;
pub mod command;
pub mod configs;
pub mod console;
pub mod desktop;
pub mod i18n;
pub mod instance;
pub mod lane;
pub mod launch;
pub mod links;
pub mod maintenance;
pub mod network;
pub mod news;
pub mod notifications;
pub mod packages;
pub mod page_ui;
pub mod pages;
pub mod panics;
pub mod pictures;
pub mod power;
pub mod profiling;
pub mod reader;
pub mod schemes;
#[cfg(feature = "selftest")]
pub mod selftest;
pub mod settings;
pub mod share;
mod shell;
pub mod snapshot;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
pub mod units;
pub mod users;
pub mod waking;

pub use app::{App, FIXED_VERSION, PreferencesSetup, Setup};
pub use toolbox_services::{apply, diagnostics, log_folder, page, programs, queue};
