use clap::error::ErrorKind;

use super::super::{
    BootCommand, Command, Group, GuideCommand, NetworkCommand, QueueCommand, SettingsCommand,
    TweaksCommand,
};
use super::parse;

fn command(words: &[&str]) -> Command {
    parse(words).unwrap().command.unwrap()
}

#[test]
fn a_collection_alone_lists() {
    assert!(matches!(
        command(&["tweaks"]),
        Command::Tweaks(Group(TweaksCommand::List(_)))
    ));
    assert!(matches!(
        command(&["settings"]),
        Command::Settings(Group(SettingsCommand::List(_)))
    ));
    assert!(matches!(
        command(&["guide"]),
        Command::Guide(Group(GuideCommand::List(_)))
    ));
}

#[test]
fn a_state_view_alone_shows() {
    assert!(matches!(
        command(&["network"]),
        Command::Network(Group(NetworkCommand::Show(_)))
    ));
    assert!(matches!(
        command(&["boot"]),
        Command::Boot(Group(BootCommand::Show(_)))
    ));
    assert!(matches!(
        command(&["queue"]),
        Command::Queue(Group(QueueCommand::Show(_)))
    ));
}

#[test]
fn the_flags_of_the_default_go_on_the_group() {
    let Command::Tweaks(Group(TweaksCommand::List(tweaks))) = command(&["tweaks", "--json"]) else {
        panic!("not tweaks list");
    };

    assert!(tweaks.json);

    let Command::Settings(Group(SettingsCommand::List(settings))) =
        command(&["settings", "--search", "lang"])
    else {
        panic!("not settings list");
    };

    assert!(settings.search.is_some());

    let Command::Queue(Group(QueueCommand::Show(read))) = command(&["queue", "--json"]) else {
        panic!("not queue show");
    };

    assert!(read.json);
}

#[test]
fn a_named_subcommand_still_wins() {
    assert!(matches!(
        command(&["guide", "show", "kernels"]),
        Command::Guide(Group(GuideCommand::Show(_)))
    ));
    assert!(matches!(
        command(&["boot", "entries", "--json"]),
        Command::Boot(Group(BootCommand::Entries(read))) if read.json
    ));
}

#[test]
fn a_flag_of_the_default_never_goes_with_another_subcommand() {
    let error = parse(&["tweaks", "--json", "dns", "cloudflare"]).unwrap_err();

    assert_eq!(error.kind(), ErrorKind::ArgumentConflict);
    assert!(error.to_string().contains("--json"), "{error}");
}

#[test]
fn the_output_flags_still_go_anywhere_in_a_group() {
    let cli = parse(&["--color", "never", "tweaks", "--width", "50", "list"]).unwrap();

    assert!(matches!(
        cli.command,
        Some(Command::Tweaks(Group(TweaksCommand::List(_))))
    ));
    assert_eq!(cli.width, Some(50));
}
