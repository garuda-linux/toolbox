use toolbox_core::features::configs::Refusal;
use toolbox_core::{ChangeSet, Snapshot};

use crate::command::{AppCommand, ConfigsCommand};
use crate::configs::Configs;
use crate::page::PageId;
use crate::page_service::PageService as _;

fn click(configs: &mut Configs, command: ConfigsCommand) -> crate::page_service::Handled {
    configs.receive(
        &AppCommand::Configs(command),
        &ChangeSet::default(),
        &Snapshot::default(),
    )
}

#[test]
fn an_unknown_row_is_refused_with_its_key_until_dismissed() {
    let mut configs = Configs::default();

    let handled = click(&mut configs, ConfigsCommand::Edit("nope".to_owned()));

    assert!(handled.taken);
    assert_eq!(handled.answers, []);
    let refused = configs.refusal().unwrap();
    assert_eq!(refused.refusal, Refusal::Unknown);
    assert_eq!(refused.subject, "nope");

    let dismissed = configs.receive(
        &AppCommand::DismissRefusal(PageId::Configs),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(dismissed.taken);
    assert!(configs.refusal().is_none());
}

#[test]
fn an_open_and_a_view_of_an_unknown_row_are_refused_too() {
    let mut configs = Configs::default();

    click(&mut configs, ConfigsCommand::Open("home".to_owned()));
    assert_eq!(
        configs.refusal().map(|refused| refused.subject.as_str()),
        Some("home")
    );

    click(&mut configs, ConfigsCommand::View("passwd".to_owned()));
    assert_eq!(
        configs.refusal().map(|refused| refused.subject.as_str()),
        Some("passwd")
    );
}

/// Without an edit there is nothing to apply, so nothing runs.
#[test]
fn apply_without_an_edit_is_taken_and_runs_nothing() {
    let mut configs = Configs::default();

    let handled = click(&mut configs, ConfigsCommand::Apply);

    assert!(handled.taken);
    assert_eq!(handled.answers, []);
}

/// The Console is App's.
#[test]
fn show_console_is_left_to_app() {
    let mut configs = Configs::default();

    let handled = click(&mut configs, ConfigsCommand::ShowConsole);

    assert!(!handled.taken);
}
