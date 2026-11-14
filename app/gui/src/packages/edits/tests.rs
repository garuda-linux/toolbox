use toolbox_core::features::packages::queue::Refusal;
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_plan::PackageName;

use crate::command::{AppCommand, PackagesCommand};
use crate::packages::Packages;
use crate::page::PageId;
use crate::page_service::PageService as _;

fn install(name: &str) -> AppCommand {
    AppCommand::Packages(PackagesCommand::Toggle {
        package: PackageName::new(name).unwrap(),
        wish: Toggle::On,
    })
}

/// Before the packages are read nothing is queued, so no wrong wish runs.
#[test]
fn an_install_before_the_read_is_refused_with_the_package() {
    let mut packages = Packages::default();

    let handled = packages.handle(&install("vlc"), &ChangeSet::default(), &Snapshot::default());

    assert!(handled.edit.is_none());
    let refused = packages.refusal().unwrap();
    assert_eq!(refused.refusal, Refusal::Unread);
    assert_eq!(refused.subject.as_str(), "vlc");

    let dismissed = packages.receive(
        &AppCommand::DismissRefusal(PageId::Packages),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(dismissed.taken);
    assert!(packages.refusal().is_none());
}

#[test]
fn a_shown_package_is_the_services() {
    let mut packages = Packages::default();
    let vlc = PackageName::new("vlc").unwrap();

    let handled = packages.handle(
        &AppCommand::Packages(PackagesCommand::Show(vlc.clone())),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(handled.taken);
    assert_eq!(packages.details().map(|shown| &shown.package), Some(&vlc));
}

/// Home's counts change with the packages, and App holds their readers.
#[test]
fn refresh_is_left_to_app() {
    let mut packages = Packages::default();

    let handled = packages.handle(
        &AppCommand::Packages(PackagesCommand::Refresh),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(!handled.taken);
}
