use super::Search;
use crate::args::tests::parse;
use crate::args::{
    AppsCommand, Command, KernelsCommand, LocalesCommand, ServicesCommand, ToggleCommand,
};

#[test]
fn a_search_ignores_the_case() {
    let search = Search::new("SSH");

    assert!(search.matches(["sshd.service"]));
    assert!(search.matches(["bluetooth.service", "OpenSSH Daemon"]));
    assert!(!search.matches(["bluetooth.service", "Bluetooth"]));
}

#[test]
fn an_empty_search_matches_everything() {
    assert!(Search::new("").matches(["anything"]));
}

#[test]
fn every_list_takes_a_search() {
    let lists: [&[&str]; 14] = [
        &["services", "list"],
        &["toggle", "list"],
        &["apps", "list"],
        &["kernels", "list"],
        &["maintenance", "list"],
        &["dkms", "list"],
        &["locales", "list"],
        &["locales", "packs", "list"],
        &["users", "list"],
        &["groups", "list"],
        &["configs", "list"],
        &["network", "drivers", "list"],
        &["setup", "list"],
        &["settings", "list"],
    ];

    for list in lists {
        let argv: Vec<&str> = list.iter().copied().chain(["--search", "x"]).collect();
        parse(&argv).unwrap_or_else(|error| panic!("{list:?}: {error}"));
    }
}

#[test]
fn services_filter_by_state_and_boot() {
    let cli = parse(&[
        "services",
        "list",
        "--running",
        "--enabled",
        "--search",
        "SSH",
    ])
    .unwrap();

    let Some(Command::Services(ServicesCommand::List(list))) = cli.command else {
        panic!("not a services list");
    };

    assert!(list.running && list.enabled && !list.failed);
    assert_eq!(list.read.search, Some(Search::new("ssh")));
}

#[test]
fn a_service_is_running_or_failed_not_both() {
    parse(&["services", "list", "--running", "--failed"]).unwrap_err();
}

#[test]
fn a_switch_is_on_or_off_not_both() {
    let cli = parse(&["toggle", "list", "--on"]).unwrap();
    assert!(matches!(
        cli.command,
        Some(Command::Toggle(ToggleCommand::List(list))) if list.on && !list.off
    ));

    parse(&["toggle", "list", "--on", "--off"]).unwrap_err();
}

#[test]
fn installed_and_enabled_filters_parse() {
    let apps = parse(&["apps", "list", "--installed"]).unwrap();
    assert!(matches!(
        apps.command,
        Some(Command::Apps(AppsCommand::List(list))) if list.installed
    ));

    let kernels = parse(&["kernels", "list", "--installed", "--json"]).unwrap();
    assert!(matches!(
        kernels.command,
        Some(Command::Kernels(KernelsCommand::List(list))) if list.installed && list.read.json
    ));

    let locales = parse(&["locales", "list", "--enabled"]).unwrap();
    assert!(matches!(
        locales.command,
        Some(Command::Locales(LocalesCommand::List(list))) if list.enabled
    ));
}

#[test]
fn no_pager_works_after_the_command() {
    assert!(parse(&["status", "--no-pager"]).unwrap().no_pager);
    assert!(parse(&["services", "list", "--no-pager"]).unwrap().no_pager);
    assert!(!parse(&["services", "list"]).unwrap().no_pager);
}
