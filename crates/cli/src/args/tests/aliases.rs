use clap::CommandFactory as _;

use super::super::{
    AppsCommand, Cli, Command, DriversCommand, KernelsCommand, LocalesCommand, NetworkCommand,
    PackagesCommand,
};
use super::parse;

fn command(args: &[&str]) -> Command {
    parse(args).unwrap().command.unwrap()
}

#[test]
fn packages_list_is_packages_search() {
    let Command::Packages(PackagesCommand::Search(search)) =
        command(&["packages", "list", "editor"])
    else {
        panic!("packages list is no search");
    };

    assert_eq!(search.text.as_deref(), Some("editor"));
}

#[test]
fn drivers_block_is_blacklist() {
    assert!(matches!(
        command(&["network", "drivers", "block", "r8169"]),
        Command::Network(NetworkCommand::Drivers(DriversCommand::Blacklist(_)))
    ));
    assert!(matches!(
        command(&["network", "drivers", "blacklist", "r8169"]),
        Command::Network(NetworkCommand::Drivers(DriversCommand::Blacklist(_)))
    ));
}

#[test]
fn locales_set_system_is_locales_system() {
    assert!(matches!(
        command(&["locales", "set-system", "de_DE.UTF-8"]),
        Command::Locales(LocalesCommand::System(_))
    ));
    assert!(matches!(
        command(&["locales", "system", "de_DE.UTF-8"]),
        Command::Locales(LocalesCommand::System(_))
    ));
}

#[test]
fn drivers_unblock_is_allow() {
    assert!(matches!(
        command(&["network", "drivers", "unblock", "r8169"]),
        Command::Network(NetworkCommand::Drivers(DriversCommand::Allow(_)))
    ));
    assert!(matches!(
        command(&["network", "drivers", "allow", "r8169"]),
        Command::Network(NetworkCommand::Drivers(DriversCommand::Allow(_)))
    ));
}

#[test]
fn kernels_set_default_is_kernels_default() {
    assert!(matches!(
        command(&["kernels", "set-default", "linux-lts"]),
        Command::Kernels(KernelsCommand::Default(_))
    ));
    assert!(matches!(
        command(&["kernels", "default", "linux-lts"]),
        Command::Kernels(KernelsCommand::Default(_))
    ));
}

#[test]
fn uninstall_is_remove() {
    for verb in ["uninstall", "remove"] {
        assert!(matches!(
            command(&["apps", verb, "firefox"]),
            Command::Apps(AppsCommand::Remove(_))
        ));
        assert!(matches!(
            command(&["kernels", verb, "linux-zen"]),
            Command::Kernels(KernelsCommand::Remove(_))
        ));
        assert!(matches!(
            command(&["packages", verb, "gimp"]),
            Command::Packages(PackagesCommand::Remove(_))
        ));
    }
}

#[test]
fn the_help_shows_the_new_names() {
    let mut top = Cli::command();
    top.build();

    for (path, alias) in [
        (&["packages", "search"][..], "list"),
        (&["network", "drivers", "blacklist"], "block"),
        (&["locales", "system"], "set-system"),
        (&["network", "drivers", "allow"], "unblock"),
        (&["kernels", "default"], "set-default"),
        (&["apps", "remove"], "uninstall"),
        (&["kernels", "remove"], "uninstall"),
        (&["packages", "remove"], "uninstall"),
    ] {
        let found = path
            .iter()
            .fold(&top, |command, name| command.find_subcommand(name).unwrap());

        assert!(
            found.get_visible_aliases().any(|known| known == alias),
            "{path:?} shows no {alias}"
        );
    }
}
