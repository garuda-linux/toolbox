use clap::Parser as _;
use toolbox_cli::args::Cli;

use super::{AT_ONCE, COMMANDS};

#[test]
fn every_kind_of_queued_change_has_a_command() {
    let domains = toolbox_core::domains::all().unwrap();
    let covered: Vec<&str> = COMMANDS.iter().map(|(id, _command)| *id).collect();

    for id in domains.ids() {
        assert!(
            covered.contains(&id),
            "the window queues `{id}` changes, but no command does: add one and its line in `COMMANDS`"
        );
    }
}

#[test]
fn every_listed_command_is_one_the_binary_takes() {
    for (what, command) in COMMANDS.iter().chain(AT_ONCE) {
        let argv = std::iter::once("garuda-toolbox").chain(command.iter().copied());
        let parsed = Cli::try_parse_from(argv);

        assert!(
            parsed.as_ref().is_ok_and(|cli| cli.command.is_some()),
            "`{what}`: `{}` is no command: {:?}",
            command.join(" "),
            parsed.err()
        );
    }
}
