use toolbox_plan::{PlannedAction, TargetList, UserAction};

use super::AurPackages;
use crate::SummaryCode;
use crate::domain::{Domain as _, Queued, Stage, Staged, Summary};
use crate::domains::InstalledPackages;
use crate::test_support::{package, packages};

#[test]
fn an_aur_install_is_done_once_the_package_is_installed() {
    let system = InstalledPackages(packages(&["yay-bin"]));

    assert!(AurPackages::is_satisfied(
        &package("yay-bin"),
        &Queued,
        &system
    ));
    assert!(!AurPackages::is_satisfied(
        &package("paru-bin"),
        &Queued,
        &system
    ));
}

#[test]
fn every_aur_install_goes_to_one_paru_run_as_the_user_after_the_repositories() {
    let yay = package("yay-bin");
    let paru = package("paru-bin");

    let planned = AurPackages::plan(&[(&yay, &Queued), (&paru, &Queued)]);

    assert_eq!(
        planned,
        [Staged::at(
            AurPackages::STAGE,
            PlannedAction::User(UserAction::InstallAur {
                packages: TargetList::new([paru, yay]).unwrap(),
            }),
        )]
    );
    assert!(AurPackages::STAGE > Stage::PACKAGES);
    assert!(AurPackages::STAGE < Stage::DKMS_BUILDS);
}

#[test]
fn an_aur_install_is_named_like_its_package() {
    assert_eq!(AurPackages::key(&package("yay-bin")), "yay-bin");
    assert_eq!(
        AurPackages::summary(&package("yay-bin"), &Queued),
        Summary::of(SummaryCode::AurInstall, "yay-bin")
    );
}
