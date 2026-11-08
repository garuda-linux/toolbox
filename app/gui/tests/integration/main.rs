//! One test binary for the whole app: each module drives the real `App`
//! through the kittest harness.

// Marks the whole crate as test code for clippy, so helpers may unwrap
// as the tests themselves do.
#![cfg(test)]

mod boot;
mod bootrepair;
mod changes;
mod common;
mod components;
mod configs;
mod console;
mod console_output;
mod console_sheet;
mod coverage;
mod diagnostics;
mod discover;
mod focus;
mod gaming;
mod guide;
mod home;
mod home_facts;
mod home_reads;
mod instance;
mod kernels;
mod locales;
mod maintenance;
mod motion;
mod network;
mod packages;
mod packages_aur;
mod page_scroll;
mod schemes;
mod services;
mod settings;
mod setup;
mod shell;
mod shortcuts;
mod sidebar;
mod snapshots;
mod startup;
mod status_bar;
mod system;
mod toasts;
mod toolbar;
mod tweaks;
mod users;
