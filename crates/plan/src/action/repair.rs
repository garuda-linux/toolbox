//! The procedure of a boot repair from the live ISO: the installed system
//! is mounted below the helper's own folder, GRUB's programs run inside it
//! through `chroot`, and every mount is undone again.
//!
//! Each mount and the unlock have an [`Invocation::Always`] counterpart at
//! the end, in reverse order, so a failed or cancelled repair leaves
//! nothing mounted and nothing unlocked. The only programs that write to a
//! disk are the installed system's own `grub-install` and `grub-mkconfig`.
//!
//! No argument needs a `--`: every device starts with `/dev/`, every mount
//! point with `/run/`, and the boot loader name with a letter or digit.

#[cfg(test)]
mod tests;

use super::run;
use crate::procedure::{Builtin, Command, Invocation, Program};
use crate::repair::{
    ESP_FOLDER, REPAIR_MAPPER, REPAIR_VOLUME, ROOT_SUBVOLUME, RepairMount, RepairRoot, RootLayout,
};
use crate::{GRUB_CONFIG, InstalledSystem, PartitionDevice, Password, RepairWork};

/// What the firmware of a 64 bit PC loads. Arch has no other.
const EFI_TARGET: &str = "x86_64-efi";

/// GRUB for the boot sector of a disk, on any PC.
const BIOS_TARGET: &str = "i386-pc";

pub fn repair_boot(
    system: &InstalledSystem,
    work: &RepairWork,
    passphrase: Option<&Password>,
) -> Vec<Invocation> {
    let mut steps = vec![Invocation::Builtin(Builtin::PrepareRepairMount)];
    let mut mounted = vec![RepairMount::Root];

    let device = match &system.root {
        RepairRoot::Open { device } => device.as_str(),
        RepairRoot::Locked { partition } => {
            steps.push(unlock(partition, passphrase));
            REPAIR_VOLUME
        }
    };

    steps.push(mount_root(device, system.layout));
    steps.push(Invocation::Builtin(Builtin::RequireInstalledSystem(
        work.need(),
    )));

    // Not recursive, so nothing mounted below them comes along.
    for place in RepairMount::BINDS {
        let source = format!("/{}", place.inside());

        steps.push(run(mount().args(["--bind", &source, &place.to_string()])));
        mounted.push(place);
    }

    match work {
        RepairWork::RegenerateConfig => {}
        RepairWork::InstallUefi { esp, id } => {
            steps.push(run(mount().args([
                "-t",
                "efivarfs",
                "efivarfs",
                &RepairMount::EfiVariables.to_string(),
            ])));
            steps.push(run(
                mount().args([esp.as_str(), &RepairMount::Esp.to_string()])
            ));
            mounted.extend([RepairMount::EfiVariables, RepairMount::Esp]);
            steps.push(recheck(system, work));
            steps.push(run(inside(Program::GrubInstall).args([
                format!("--target={EFI_TARGET}"),
                format!("--efi-directory=/{ESP_FOLDER}"),
                format!("--bootloader-id={id}"),
                "--recheck".to_owned(),
            ])));
        }
        RepairWork::InstallBios { disk } => {
            steps.push(recheck(system, work));
            steps.push(run(inside(Program::GrubInstall).args([
                format!("--target={BIOS_TARGET}"),
                "--recheck".to_owned(),
                disk.to_string(),
            ])));
        }
    }

    steps.push(run(
        inside(Program::GrubMkconfig).args(["-o".to_owned(), format!("/{GRUB_CONFIG}")])
    ));

    steps.extend(mounted.into_iter().rev().map(unmount));

    if system.is_locked() {
        steps.push(Invocation::Always(
            Command::new(Program::Cryptsetup).args(["close", REPAIR_MAPPER]),
        ));
    }

    steps
}

fn recheck(system: &InstalledSystem, work: &RepairWork) -> Invocation {
    Invocation::Builtin(Builtin::RecheckRepairDevices {
        system: system.clone(),
        work: work.clone(),
    })
}

/// `--key-file -` makes cryptsetup read the passphrase from its standard
/// input to the end, so it never is an argument and no terminal echoes it.
fn unlock(partition: &PartitionDevice, passphrase: Option<&Password>) -> Invocation {
    Invocation::Feed {
        command: Command::new(Program::Cryptsetup).args([
            "open",
            "--type",
            "luks",
            "--key-file",
            "-",
            partition.as_str(),
            REPAIR_MAPPER,
        ]),
        input: passphrase.map(Password::key_input),
    }
}

fn mount_root(device: &str, layout: RootLayout) -> Invocation {
    let command = match layout {
        RootLayout::Plain => mount(),
        RootLayout::Subvolume => {
            mount().args(["-o".to_owned(), format!("subvol={ROOT_SUBVOLUME}")])
        }
    };

    run(command.args([device, &RepairMount::Root.to_string()]))
}

/// Runs whatever happened before. Unmounting what was never mounted only
/// prints that it is not mounted. `-R` because a program in the chroot
/// (os-prober) may mount more below, which would keep the device busy.
fn unmount(place: RepairMount) -> Invocation {
    Invocation::Always(Command::new(Program::Umount).args(["-R".to_owned(), place.to_string()]))
}

const fn mount() -> Command {
    Command::new(Program::Mount)
}

/// `program` of the installed system, by its full path there.
fn inside(program: Program) -> Command {
    Command::new(Program::Chroot).args([RepairMount::Root.to_string(), program.path().to_owned()])
}
