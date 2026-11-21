//! One test binary for the whole app: each module drives the real `App`
//! through the kittest harness.

// Marks the whole crate as test code for clippy, so helpers may unwrap
// as the tests themselves do.
#![cfg(test)]

mod boot;
mod bootrepair;
mod captured;
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
mod guide_images;
mod home;
mod home_facts;
mod home_health;
mod home_reads;
mod instance;
mod kernels;
mod kernels_default;
mod locales;
mod look;
mod maintenance;
mod motion;
mod network;
mod packages;
mod packages_aur;
mod page_scroll;
mod palette_packages;
mod palette_sections;
mod schemes;
mod sdboot;
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
mod zoom;
