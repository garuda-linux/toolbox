//! The Boot repair page's root action against the built helper. Only what
//! is safe on any machine: a dry run, which starts no program and looks at
//! no device, and requests the helper must refuse while it decodes them.

mod common;
mod handmade;
mod repair_names;

use std::path::Path;

use common::{TestResult, events, helper, outcomes, request};
use handmade::request_line;
use repair_names::{MAPPER, MOUNT_FOLDER, PASSPHRASE};
use toolbox_plan::protocol::{Event, Mode, Outcome};
use toolbox_plan::{
    BootloaderId, DiskDevice, InstalledSystem, PartitionDevice, Password, RepairRoot, RepairWork,
    RootAction, RootLayout, VolumeDevice,
};

fn repairs() -> Result<Vec<RootAction>, Box<dyn std::error::Error>> {
    let open = InstalledSystem {
        root: RepairRoot::Open {
            device: VolumeDevice::new("/dev/sda2")?,
        },
        layout: RootLayout::Subvolume,
    };

    let locked = InstalledSystem {
        root: RepairRoot::Locked {
            partition: PartitionDevice::new("/dev/nvme0n1p2")?,
        },
        layout: RootLayout::Plain,
    };

    Ok(vec![
        RootAction::RepairBoot {
            system: open.clone(),
            work: RepairWork::RegenerateConfig { esp: None },
            passphrase: None,
        },
        RootAction::RepairBoot {
            system: open,
            work: RepairWork::InstallBios {
                disk: DiskDevice::new("/dev/sda")?,
            },
            passphrase: None,
        },
        RootAction::RepairBoot {
            system: locked,
            work: RepairWork::InstallUefi {
                esp: PartitionDevice::new("/dev/nvme0n1p1")?,
                id: BootloaderId::new("Garuda")?,
            },
            passphrase: Some(Password::new(PASSPHRASE.to_owned())?),
        },
    ])
}

#[test]
fn repairs_are_accepted_in_a_dry_run_that_touches_nothing() -> TestResult {
    let existed = Path::new(MOUNT_FOLDER).exists();
    let output = helper(&["apply"], &request(Mode::DryRun, repairs()?)?)?;
    let skipped = outcomes(&output)?;

    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(skipped, [Outcome::Skipped; 3]);
    assert_eq!(Path::new(MOUNT_FOLDER).exists(), existed);
    assert!(!Path::new(MAPPER).exists(), "a dry run opened the system");
    Ok(())
}

/// A dry run takes no secret line, and the request line never holds the
/// passphrase, so nothing of a dry run can show it.
#[test]
fn a_dry_run_of_a_locked_system_never_shows_the_passphrase() -> TestResult {
    let line = request(Mode::DryRun, repairs()?)?;
    let output = helper(&["apply"], &line)?;

    for text in [&line, &output.stdout, &output.stderr] {
        assert!(!text.contains(PASSPHRASE), "{text}");
        assert!(!text.contains("passphrase"), "{text}");
    }

    Ok(())
}

#[test]
fn a_device_that_is_no_disk_or_partition_name_rejects_the_whole_request() -> TestResult {
    let system = |device: &str| {
        format!(
            r#"{{"kind":"repair-boot","system":{{"root":{{"kind":"open","device":"{device}"}},"layout":"plain"}},"work":{{"kind":"regenerate-config"}}}}"#
        )
    };

    let work = |work: &str| {
        format!(
            r#"{{"kind":"repair-boot","system":{{"root":{{"kind":"open","device":"/dev/sda2"}},"layout":"plain"}},"work":{work}}}"#
        )
    };

    for action in [
        system("/etc/shadow"),
        system("/dev/../etc/shadow"),
        system("/dev/sda"),
        system("/dev/loop0"),
        system("/dev/mapper/../sda"),
        system("/dev/mapper/-q"),
        system("/dev/disk/by-label/root"),
        system("--force"),
        work(r#"{"kind":"install-bios","disk":"/dev/sda1"}"#),
        work(r#"{"kind":"install-bios","disk":"/home/nico/disk.img"}"#),
        work(r#"{"kind":"install-uefi","esp":"/dev/sda","id":"Garuda"}"#),
        work(r#"{"kind":"install-uefi","esp":"/dev/sda1","id":"--removable"}"#),
        work(r#"{"kind":"install-uefi","esp":"/dev/sda1","id":"a/../../b"}"#),
        work(r#"{"kind":"back-up-boot-sector","disk":"/dev/sda"}"#),
    ] {
        let output = helper(&["apply"], &request_line("dry-run", &action))?;
        let all = events(&output)?;

        assert!(!output.status.success(), "{action}");
        assert!(
            matches!(all.first(), Some(Event::Rejected { reason: _ })),
            "{action}: {}",
            output.stdout
        );
        assert!(
            !all.iter()
                .any(|event| matches!(event, Event::Started { step: _ })),
            "{action}"
        );
    }

    Ok(())
}

#[test]
fn a_layout_of_the_caller_s_choice_does_not_exist() -> TestResult {
    let action = r#"{"kind":"repair-boot","system":{"root":{"kind":"open","device":"/dev/sda2"},"layout":"subvol=@,exec"},"work":{"kind":"regenerate-config"}}"#;
    let output = helper(&["apply"], &request_line("dry-run", action))?;
    let first = events(&output)?.into_iter().next();

    assert!(
        matches!(first, Some(Event::Rejected { reason: _ })),
        "{}",
        output.stdout
    );
    Ok(())
}

/// A request that names its own mapper or mount point is refused whole:
/// the helper picks both itself.
#[test]
fn a_mapper_or_mount_point_of_the_caller_s_choice_is_refused() -> TestResult {
    let chosen = r#"{"kind":"repair-boot","system":{"root":{"kind":"locked","partition":"/dev/sda2","mapper":"../x"},"layout":"plain","mount":"/"},"work":{"kind":"regenerate-config"}}"#;
    let plain = r#"{"kind":"repair-boot","system":{"root":{"kind":"locked","partition":"/dev/sda2"},"layout":"plain"},"work":{"kind":"regenerate-config"}}"#;

    let with = helper(&["apply"], &request_line("dry-run", chosen))?;
    let without = helper(&["apply"], &request_line("dry-run", plain))?;

    assert!(
        !with.status.success(),
        "a field the request has not was taken"
    );
    assert!(without.status.success(), "{}", without.stderr);
    assert_ne!(events(&with)?, events(&without)?);
    Ok(())
}
