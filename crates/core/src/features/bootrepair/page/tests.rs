use toolbox_plan::{DiskDevice, PartitionDevice, RootLayout};

use super::{Facts, Offer, Progress, State, layout, state, system, target, targets, usual_target};
use crate::features::bootrepair::{
    Firmware, RepairFacts, SAMPLE_LISTING, Target, Tool, parse_lsblk,
};

fn facts(firmware: Firmware) -> RepairFacts {
    RepairFacts {
        firmware,
        missing: Vec::new(),
        inventory: parse_lsblk(SAMPLE_LISTING).unwrap(),
    }
}

fn esp(device: &str) -> Target {
    Target::Esp(PartitionDevice::new(device).unwrap())
}

#[test]
fn the_page_shows_on_the_live_iso_or_with_the_flag() {
    assert_eq!(Offer::of(true, false), Offer::Live);
    assert_eq!(Offer::of(true, true), Offer::Live);
    assert_eq!(Offer::of(false, true), Offer::Test);
    assert_eq!(Offer::of(false, false), Offer::Hidden);
}

#[test]
fn the_state_follows_the_read_and_a_hidden_page_knows_nothing() {
    let known = Facts::Known(facts(Firmware::Uefi));

    assert_eq!(state(Offer::Hidden, &known), State::Hidden);
    assert_eq!(known.known(Offer::Hidden), None);
    assert_eq!(state(Offer::Live, &Facts::NotAsked), State::Loading);
    assert_eq!(state(Offer::Live, &Facts::Loading), State::Loading);
    assert_eq!(state(Offer::Live, &Facts::Failed), State::Failed);
    assert_eq!(state(Offer::Test, &known), State::Ready);

    let missing = Facts::Known(RepairFacts {
        missing: vec![Tool::GrubInstall],
        ..facts(Firmware::Bios)
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
    let facts = facts(Firmware::Uefi);
    let first = system(&facts, None).unwrap();

    assert_eq!(first.device(), "/dev/sda2");
    assert_eq!(usual_target(&facts, first), Some(esp("/dev/sda1")));
    assert_eq!(
        target(&facts, None, Some(first)).map(|choice| choice.target),
        Some(esp("/dev/sda1"))
    );
    assert_eq!(layout(first, None), Some(RootLayout::Subvolume));

    let locked = system(&facts, Some("/dev/sda3")).unwrap();

    assert_eq!(
        layout(locked, Some(RootLayout::Plain)),
        Some(RootLayout::Plain)
    );
    assert_eq!(
        system(&facts, Some("/dev/nothing")).unwrap().device(),
        "/dev/sda2"
    );
}

#[test]
fn a_bios_machine_offers_free_disks_and_ignores_an_unlisted_pick() {
    let facts = facts(Firmware::Bios);
    let disk = Target::Disk(DiskDevice::new("/dev/sda").unwrap());
    let listed = targets(&facts);

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].target, disk);
    assert_eq!(listed[0].label, "Samsung SSD 870 EVO 500GB");
    assert_eq!(
        target(&facts, Some(&esp("/dev/sda1")), system(&facts, None)).map(|choice| choice.target),
        Some(disk)
    );
}

#[test]
fn a_target_names_its_device() {
    let disk = Target::Disk(DiskDevice::new("/dev/sda").unwrap());

    assert_eq!(esp("/dev/sda1").device(), "/dev/sda1");
    assert_eq!(disk.device(), "/dev/sda");
}
