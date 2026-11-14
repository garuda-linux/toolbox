use toolbox_core::domains::CurrentDns;
use toolbox_core::features::toggles::Refusal;
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::{DnsProvider, ShellPath};

use super::TogglePages;
use crate::catalogs::CatalogId;
use crate::command::{AppCommand, DiscoverCommand, TweakCommand};
use crate::page::PageId;
use crate::page_service::PageService as _;
use crate::refused::Refused;

#[test]
fn each_page_keeps_its_own_refusal_until_dismissed() {
    let mut pages = TogglePages::default();
    let unmet = Refused::new(Refusal::Unmet, "printing".to_owned());
    let unknown = Refused::new(Refusal::Unknown, "nope".to_owned());
    pages.set_refusal(CatalogId::Gaming, Some(unmet));
    pages.set_refusal(CatalogId::Setup, Some(unknown.clone()));

    let handled = pages.receive(
        &AppCommand::DismissRefusal(PageId::Gaming),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(handled.taken);
    assert_eq!(pages.refusal(CatalogId::Gaming), None);
    assert_eq!(pages.refusal(CatalogId::Setup), Some(&unknown));
}

/// System Tweaks shows the settings catalog, whose page has another name.
#[test]
fn the_tweaks_page_dismisses_the_settings_refusal() {
    let mut pages = TogglePages::default();
    let unmet = Refused::new(Refusal::Unmet, "printing".to_owned());
    pages.set_refusal(CatalogId::Settings, Some(unmet));

    let handled = pages.receive(
        &AppCommand::DismissRefusal(PageId::SystemSettings),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(handled.taken);
    assert_eq!(pages.refusal(CatalogId::Settings), None);
}

#[test]
fn another_page_s_dismissal_is_not_taken() {
    let mut pages = TogglePages::default();

    let handled = pages.receive(
        &AppCommand::DismissRefusal(PageId::Kernels),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(!handled.taken);
}

/// Without a loaded catalog the click is the page's all the same, so App
/// does not run it a second time.
#[test]
fn a_click_on_an_unloaded_catalog_is_taken_and_changes_nothing() {
    let mut pages = TogglePages::default();

    let handled = pages.handle(
        &AppCommand::ToggleEntry {
            catalog: CatalogId::Gaming,
            entry: "steam".to_owned(),
            wish: toolbox_core::Toggle::On,
        },
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(handled.taken);
    assert!(handled.edit.is_none());
}

/// Discover's switches meet the same guards. A refused one stays on the
/// page with its reason until dismissed, instead of a line in the log.
#[test]
fn a_refused_app_click_keeps_its_reason_until_dismissed() {
    let mut pages = TogglePages::default();

    let handled = pages.handle(
        &AppCommand::Discover(DiscoverCommand::Toggle {
            app: "nope".to_owned(),
            wish: toolbox_core::Toggle::On,
        }),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(handled.taken);
    assert!(handled.edit.is_none());
    assert_eq!(
        pages.app_refusal().map(|refused| refused.refusal),
        Some(Refusal::Unknown)
    );

    let dismissed = pages.receive(
        &AppCommand::DismissRefusal(PageId::Discover),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(dismissed.taken);
    assert_eq!(pages.app_refusal(), None);
}

#[test]
fn a_reveal_is_no_click() {
    let mut pages = TogglePages::default();

    let handled = pages.handle(
        &AppCommand::Discover(DiscoverCommand::Reveal("Firefox".to_owned())),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(!handled.taken);
}

#[test]
fn a_dns_choice_is_queued() {
    let mut pages = TogglePages::default();
    let mut snapshot = Snapshot::default();
    snapshot.insert(CurrentDns(DnsProvider::Default));

    let handled = pages.handle(
        &AppCommand::Tweak(TweakCommand::Dns(DnsProvider::Quad9)),
        &ChangeSet::default(),
        &snapshot,
    );

    assert!(handled.taken);
    assert!(handled.edit.is_some_and(|edit| edit.len() == 1));
}

/// A wish the system already fulfils is not queued.
#[test]
fn the_dns_the_system_uses_is_not_queued() {
    let mut pages = TogglePages::default();
    let mut snapshot = Snapshot::default();
    snapshot.insert(CurrentDns(DnsProvider::Quad9));

    let handled = pages.handle(
        &AppCommand::Tweak(TweakCommand::Dns(DnsProvider::Quad9)),
        &ChangeSet::default(),
        &snapshot,
    );

    assert!(handled.edit.is_some_and(|edit| edit.is_empty()));
}

/// Before the probe names the account there is nobody to give the shell.
#[test]
fn a_shell_before_the_account_is_known_is_taken_and_queues_nothing() {
    let mut pages = TogglePages::default();

    let handled = pages.handle(
        &AppCommand::Tweak(TweakCommand::Shell(ShellPath::new("/bin/fish").unwrap())),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(handled.taken);
    assert!(handled.edit.is_none());
}
