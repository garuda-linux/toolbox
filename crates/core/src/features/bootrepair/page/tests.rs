use toolbox_plan::{DiskDevice, RootLayout, Toggle};

use super::{
    Booted, Facts, Offer, Progress, State, layout, state, system, target, targets, usual_target,
};
use crate::features::bootrepair::tests::listings::{BARE_METAL, laptop_from_the_live_iso};
use crate::features::bootrepair::tests::{esp, facts};
use crate::features::bootrepair::{Firmware, RepairFacts, Target, Tool};

#[test]
fn the_page_shows_on_the_live_iso_or_with_the_flag() {
    assert_eq!(Offer::of(Booted::Live, Toggle::Off), Offer::Live);
    assert_eq!(Offer::of(Booted::Live, Toggle::On), Offer::Live);
    assert_eq!(Offer::of(Booted::Installed, Toggle::On), Offer::Test);
    assert_eq!(Offer::of(Booted::Installed, Toggle::Off), Offer::Hidden);
}

#[test]
fn the_state_follows_the_read_and_a_hidden_page_knows_nothing() {
    let known = Facts::Known(facts(BARE_METAL, Firmware::Uefi));

    assert_eq!(state(Offer::Hidden, &known), State::Hidden);
    assert_eq!(known.known(Offer::Hidden), None);
    assert_eq!(state(Offer::Live, &Facts::NotAsked), State::Loading);
    assert_eq!(state(Offer::Live, &Facts::Loading), State::Loading);
    assert_eq!(state(Offer::Live, &Facts::Failed), State::Failed);
    assert_eq!(state(Offer::Test, &known), State::Ready);

    let missing = Facts::Known(RepairFacts {
        missing: vec![Tool::GrubInstall],
        ..facts(BARE_METAL, Firmware::Bios)
    });

    assert_eq!(state(Offer::Live, &missing), State::Missing);
}

#[test]
fn only_the_run_this_page_started_takes_an_outcome() {
    assert_eq!(Progress::Waiting.started(), Progress::Running);
    assert_eq!(Progress::Waiting.refused(), Progress::Busy);
    assert_eq!(Progress::Running.finished(true), Progress::Succeeded);
    assert_eq!(Progress::Running.finished(false), Progress::Failed);
    assert_eq!(Progress::None.finished(true), Progress::None);
    assert_eq!(Progress::Succeeded.started(), Progress::Succeeded);
    assert_eq!(Progress::Failed.forgotten(), Progress::None);
    assert_eq!(Progress::Running.forgotten(), Progress::Running);
    assert_eq!(Progress::Waiting.as_str(), "running");
    assert_eq!(Progress::Busy.as_str(), "busy");
}

#[test]
fn the_pick_falls_back_to_the_first_free_system_and_its_target() {
    let facts = facts(&laptop_from_the_live_iso(), Firmware::Uefi);
    let first = system(&facts, None).unwrap();

    assert_eq!(first.device(), "/dev/nvme0n1p1");
    assert_eq!(usual_target(&facts, first), Some(esp("/dev/nvme0n1p2")));
    assert_eq!(layout(first, None), Some(RootLayout::Subvolume));
    assert_eq!(
        layout(first, Some(RootLayout::Plain)),
        Some(RootLayout::Plain)
    );

    let other = system(&facts, Some("/dev/nvme1n1p2")).unwrap();

    assert_eq!(
        target(&facts, None, Some(other)).map(|choice| choice.target),
        Some(esp("/dev/nvme1n1p1"))
    );
    assert_eq!(layout(other, None), Some(RootLayout::Subvolume));
    assert_eq!(
        system(&facts, Some("/dev/nothing")).unwrap().device(),
        "/dev/nvme0n1p1"
    );
}

#[test]
fn a_bios_machine_offers_free_disks_and_ignores_an_unlisted_pick() {
    let facts = facts(BARE_METAL, Firmware::Bios);
    let disk = Target::Disk(DiskDevice::new("/dev/nvme1n1").unwrap());
    let listed = targets(&facts);

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].target, disk);
    assert_eq!(listed[0].label, "MTFDKBA1T0QGN-1BN1AABGA");
    assert_eq!(
        target(&facts, Some(&esp("/dev/nvme1n1p1")), system(&facts, None))
            .map(|choice| choice.target),
        Some(disk)
    );
}

#[test]
fn a_target_names_its_device() {
    let disk = Target::Disk(DiskDevice::new("/dev/nvme1n1").unwrap());

    assert_eq!(esp("/dev/nvme1n1p1").device(), "/dev/nvme1n1p1");
    assert_eq!(disk.device(), "/dev/nvme1n1");
}
