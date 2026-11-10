//! Every domain's command, typed after `queue add`, lands a change in
//! that domain, so the command line keeps up with the window.

mod system;

use std::path::Path;

use clap::Parser as _;
use toolbox_cli::args::Cli;
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_services::catalogs::Catalogs;

use super::Copy;
use crate::parity::COMMANDS;
use crate::queue::add::queueable;
use crate::queue::build::{Sources, build};

fn catalogs() -> Catalogs {
    Catalogs::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog"))
}

/// The install a domain's command changes something on.
fn system_for(domain: &str) -> Snapshot {
    match domain {
        "loader-settings" | "sdboot-kernel" => system::sdboot_install(),
        "system-locale" => system::german_enabled(),
        _ => system::grub_install(),
    }
}

#[test]
fn every_domains_command_queues_a_change_of_that_domain() {
    let catalogs = catalogs();
    let sources = Sources {
        catalogs: &catalogs,
        system: &system::reads(),
        aur: &Copy::default(),
    };

    let mut failed = Vec::new();

    for (domain, words) in COMMANDS {
        let argv = std::iter::once("garuda-toolbox").chain(words.iter().copied());
        let command = Cli::try_parse_from(argv).unwrap().command.unwrap();
        let change = queueable(&command).unwrap();
        let mut changes = ChangeSet::default();

        match build(&change, &sources, &system_for(domain), &mut changes) {
            Ok(()) if changes.holds(domain) => {}
            Ok(()) => failed.push(format!("{domain}: queued {:?}", changes.views())),
            Err(refused) => failed.push(format!("{domain}: refused {refused:?}")),
        }
    }

    assert!(failed.is_empty(), "{}", failed.join("\n"));
}
