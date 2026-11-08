//! The boot repair in the Arch test image, as root. Only refusals: the
//! helper looks at the named devices before any program runs, and a
//! request for a device that is not there must end right there. Nothing
//! here mounts, unlocks or writes to a disk.

use std::path::Path;

use toolbox_plan::protocol::{Event, Mode};
use toolbox_plan::{
    DiskDevice, InstalledSystem, PartitionDevice, Password, RepairRoot, RepairWork, RootAction,
    RootLayout, VolumeDevice,
};

use super::common::{TestResult, events, helper, request};
use super::in_test_image;
use crate::repair_names::{MAPPER, MOUNT_FOLDER, PASSPHRASE};

/// Names of the right shape that no machine has.
const ABSENT_VOLUME: &str = "/dev/mapper/toolbox-test-absent";
const ABSENT_PARTITION: &str = "/dev/vdzzz999";
const ABSENT_DISK: &str = "/dev/vdzzz";

fn absent() -> bool {
    [ABSENT_VOLUME, ABSENT_PARTITION, ABSENT_DISK]
        .iter()
        .all(|path| !Path::new(path).exists())
}

fn refused(action: RootAction, secret: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    let mut input = request(Mode::Run, vec![action])?;

    if let Some(secret) = secret {
        input = format!("{input}secret 1 {secret}\n");
    }

    let output = helper(&["apply"], &input)?;
    let all = events(&output)?;

    assert!(!output.status.success(), "{}", output.stdout);
    assert!(
        !all.iter()
            .any(|event| matches!(event, Event::Started { step: _ })),
        "a step started: {}",
        output.stdout
    );
    assert!(!output.stdout.contains(PASSPHRASE), "{}", output.stdout);
    assert!(!output.stderr.contains(PASSPHRASE), "{}", output.stderr);

    match all.into_iter().next() {
        Some(Event::Rejected { reason }) => Ok(reason),
        other => Err(format!("not rejected: {other:?}").into()),
    }
}

#[test]
fn a_repair_of_a_device_that_is_not_there_runs_nothing() -> TestResult {
    if !in_test_image() || !absent() {
        return Ok(());
    }

    let existed = Path::new(MOUNT_FOLDER).exists();
    let reason = refused(
        RootAction::RepairBoot {
            system: InstalledSystem {
                root: RepairRoot::Open {
                    device: VolumeDevice::new(ABSENT_VOLUME)?,
                },
                layout: RootLayout::Subvolume,
            },
            work: RepairWork::InstallBios {
                disk: DiskDevice::new(ABSENT_DISK)?,
            },
            passphrase: None,
        },
        None,
    )?;

    assert_eq!(reason, format!("{ABSENT_VOLUME} is no block device"));
    assert_eq!(Path::new(MOUNT_FOLDER).exists(), existed);
    assert!(!Path::new(MAPPER).exists());
    Ok(())
}

#[test]
fn a_locked_partition_that_is_not_there_is_refused_without_showing_the_passphrase() -> TestResult {
    if !in_test_image() || !absent() {
        return Ok(());
    }

    let reason = refused(
        RootAction::RepairBoot {
            system: InstalledSystem {
                root: RepairRoot::Locked {
                    partition: PartitionDevice::new(ABSENT_PARTITION)?,
                },
                layout: RootLayout::Plain,
            },
            work: RepairWork::RegenerateConfig { esp: None },
            passphrase: Some(Password::new(PASSPHRASE.to_owned())?),
        },
        Some(PASSPHRASE),
    )?;

    assert_eq!(reason, format!("{ABSENT_PARTITION} is no block device"));
    assert!(!Path::new(MAPPER).exists());
    Ok(())
}
