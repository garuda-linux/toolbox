use clap::Parser as _;
use toolbox_cli::args::Cli;

use super::json_read;

fn read(args: &[&str]) -> bool {
    let cli =
        Cli::try_parse_from(std::iter::once("garuda-toolbox").chain(args.iter().copied())).unwrap();

    json_read(&cli.command.unwrap())
}

#[test]
fn a_read_with_json_is_one() {
    assert!(read(&["status", "--json"]));
    assert!(read(&["services", "list", "--json"]));
    assert!(read(&["settings", "get", "language", "--json"]));
    assert!(read(&["network", "public-address", "--json"]));
}

#[test]
fn text_and_changes_are_not() {
    assert!(!read(&["status"]));
    assert!(!read(&[
        "services",
        "start",
        "sshd.service",
        "--json",
        "--yes"
    ]));
}
