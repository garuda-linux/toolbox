use toolbox_core::features::diagnostics::Selection;
use toolbox_core::{ChangeSet, Snapshot};

use crate::bootrepair::BootRepair;
use crate::command::{AppCommand, DiagnosticsCommand};
use crate::diagnostics::Diagnostics;
use crate::page::PageId;
use crate::page_service::{Answer, PageService};

fn receive(service: &mut dyn PageService, command: &AppCommand) -> bool {
    service
        .receive(command, &ChangeSet::default(), &Snapshot::default())
        .taken
}

/// Diagnostics needs nothing of App for its clicks, so App never runs one.
#[test]
fn every_diagnostics_click_is_the_services() {
    let mut diagnostics = Diagnostics::default();

    for command in [
        DiagnosticsCommand::Collect(Selection::All),
        DiagnosticsCommand::Cancel,
        DiagnosticsCommand::Upload,
        DiagnosticsCommand::HideShare,
    ] {
        assert!(receive(&mut diagnostics, &AppCommand::Diagnostics(command)));
    }
}

/// A page without refusals leaves the dismissal of another page alone.
#[test]
fn a_service_without_refusals_takes_no_dismissal() {
    let mut diagnostics = Diagnostics::default();

    assert!(!receive(
        &mut diagnostics,
        &AppCommand::DismissRefusal(PageId::Diagnostics)
    ));
}

fn live_snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(toolbox_system::Host {
        user: None,
        live_iso: true,
        desktops: Vec::new(),
        edition: None,
        reboot_pending: false,
    });

    snapshot
}

fn offers(answers: &[AppCommand]) -> Vec<bool> {
    answers
        .iter()
        .filter_map(|answer| {
            if let AppCommand::Answer(Answer::OfferBootRepair(offered)) = answer {
                Some(*offered)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn the_sidebar_hears_of_the_boot_repair_offer_only_when_it_changes() {
    let mut repair = BootRepair::default();
    let live = live_snapshot();

    assert_eq!(
        offers(&PageService::take(&mut repair, PageId::Home, &live)),
        [true]
    );
    assert_eq!(
        offers(&PageService::take(&mut repair, PageId::Home, &live)),
        [false; 0]
    );
    assert_eq!(
        offers(&PageService::take(
            &mut repair,
            PageId::Home,
            &Snapshot::default()
        )),
        [false]
    );
}
