use clap::Parser as _;
use toolbox_cli::args::Cli;

use super::{AT_ONCE, COMMANDS, NO_COMMAND};

/// Every domain has a command or a reason why not, and no line names a
/// domain that is gone.
#[test]
fn every_kind_of_queued_change_has_a_command() {
    let mut listed: Vec<&str> = COMMANDS
        .iter()
        .map(|(id, _command)| *id)
        .chain(NO_COMMAND.iter().map(|(id, _why)| *id))
        .collect();

    listed.sort_unstable();

    assert_eq!(
        listed,
        toolbox_core::domains::all().unwrap().ids(),
        "each domain needs its line in `COMMANDS`, or in `NO_COMMAND` with why"
    );
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
