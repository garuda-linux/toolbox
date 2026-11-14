use toolbox_core::features::diagnostics::Selection;
use toolbox_core::{ChangeSet, Snapshot};

use crate::command::{AppCommand, DiagnosticsCommand};
use crate::diagnostics::Diagnostics;
use crate::page::PageId;
use crate::page_service::PageService;

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
