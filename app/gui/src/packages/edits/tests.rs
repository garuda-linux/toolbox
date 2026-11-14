use toolbox_core::features::packages::queue::Refusal;
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_plan::PackageName;

use crate::command::{AppCommand, PackagesCommand};
use crate::packages::Packages;

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

    packages.handle(
        &AppCommand::Packages(PackagesCommand::DismissRefusal),
        &ChangeSet::default(),
        &Snapshot::default(),
    );
    assert!(packages.refusal().is_none());
}
