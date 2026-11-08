//! The procedure of a boot repair from the live ISO: the installed system
//! is mounted below the helper's own folder, its boot loader's programs run
//! inside it through `chroot`, and every mount is undone again.
//!
//! The root, each bind and the unlock have an [`Invocation::Always`]
//! counterpart at the end, in reverse order, so a failed or cancelled
//! repair leaves nothing mounted and nothing unlocked. The only programs
//! that write to a disk are the installed system's own boot loader
//! installer, initramfs builder and menu or image builder.
//!
//! No argument needs a `--`: every device starts with `/dev/`, every mount
//! point with `/run/`, and the boot loader name with a letter or digit.

#[cfg(test)]
mod tests;

use std::fmt;

use super::run;
use crate::procedure::{Builtin, Command, InitramfsRoot, Invocation, Program, relaxed};
use crate::repair::{
    ESP_FOLDER, EspPlace, InstalledLoader, REPAIR_MAPPER, REPAIR_VOLUME, ROOT_SUBVOLUME,
    RepairMount, RepairRoot, RootLayout,
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

    // The ESP and efivarfs it mounts lie below the root and `sys`, so the
    // `umount -R` of those takes them along.
    steps.push(Invocation::Builtin(Builtin::RepairBootLoader {
        system: system.clone(),
        work: work.clone(),
    }));
    steps.extend(mounted.into_iter().rev().map(unmount));

    if system.is_locked() {
        steps.push(Invocation::Always(
            Command::new(Program::Cryptsetup).args(["close", REPAIR_MAPPER]),
        ));
    }

    steps
}

/// What the helper runs once it has seen which boot loader the mounted
/// system starts with.
///
/// # Errors
///
/// The reason, in words for the Console, when the work does not fit the
/// system's boot loader.
pub fn boot_loader_steps(
    system: &InstalledSystem,
    work: &RepairWork,
    loader: InstalledLoader,
) -> Result<Vec<Invocation>, &'static str> {
    match loader {
        InstalledLoader::Grub => Ok(grub_steps(system, work)),
        InstalledLoader::SystemdBoot { esp: Some(place) } => sdboot_steps(system, work, place),
        InstalledLoader::SystemdBoot { esp: None } => Err(
            "the system's /etc/fstab mounts no EFI system partition at /efi, /boot or /boot/efi",
        ),
    }
}

pub fn grub_steps(system: &InstalledSystem, work: &RepairWork) -> Vec<Invocation> {
    let mut steps = Vec::new();

    match work {
        RepairWork::RegenerateConfig { esp: _ } => {}
        RepairWork::InstallUefi { esp, id } => {
            steps.extend(esp_mounts(esp, EspPlace::BootEfi, true));
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

    steps.extend(boot_files());
    steps
}

/// The images live on the ESP, so even a rebuild alone mounts it.
pub fn sdboot_steps(
    system: &InstalledSystem,
    work: &RepairWork,
    place: EspPlace,
) -> Result<Vec<Invocation>, &'static str> {
    let mut steps = Vec::new();

    match work {
        RepairWork::RegenerateConfig { esp: Some(esp) } => {
            steps.extend(esp_mounts(esp, place, false));
            steps.push(recheck(system, work));
        }
        RepairWork::RegenerateConfig { esp: None } => {
            return Err(
                "systemd-boot builds its images onto the EFI system partition, and the repair names none",
            );
        }
        RepairWork::InstallUefi { esp, id: _ } => {
            steps.extend(esp_mounts(esp, place, true));
            steps.push(recheck(system, work));
            steps.push(run(relaxed(inside(Program::Bootctl).args([
                "install".to_owned(),
                format!("--esp-path=/{}", place.inside()),
            ]))));
        }
        RepairWork::InstallBios { disk: _ } => {
            return Err("systemd-boot starts only through UEFI");
        }
    }

    steps.extend([
        Invocation::Builtin(Builtin::RebuildInitramfs {
            root: InitramfsRoot::Repair,
        }),
        Invocation::Builtin(Builtin::RebuildUkis {
            root: InitramfsRoot::Repair,
        }),
    ]);
    Ok(steps)
}

/// efivarfs is only needed where the firmware gets a boot entry.
fn esp_mounts(esp: &PartitionDevice, place: EspPlace, variables: bool) -> Vec<Invocation> {
    let mut steps = Vec::new();

    if variables {
        steps.push(run(mount().args([
            "-t",
            "efivarfs",
            "efivarfs",
            &RepairMount::EfiVariables.to_string(),
        ])));
    }

    steps.push(run(
        mount().args([esp.as_str(), &RepairMount::Esp(place).to_string()])
    ));
    steps
}

/// Both lists, since the boot loader shows only once the system is
/// mounted. The systemd-boot one shows the ESP at `/efi`, its place on
/// Garuda.
pub fn describe_loader(
    out: &mut fmt::Formatter<'_>,
    system: &InstalledSystem,
    work: &RepairWork,
) -> fmt::Result {
    let joined = |steps: &[Invocation]| {
        steps
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", then ")
    };

    write!(out, "on GRUB: {}", joined(&grub_steps(system, work)))?;

    match sdboot_steps(system, work, EspPlace::Efi) {
        Ok(steps) => write!(
            out,
            ", on systemd-boot: {}  # the ESP goes where the system's /etc/fstab mounts it",
            joined(&steps)
        ),
        Err(reason) => write!(out, ", on systemd-boot: nothing, {reason}"),
    }
}

/// grub-mkconfig writes an initrd line only for an initramfs it finds, so
/// the rebuild comes first.
fn boot_files() -> [Invocation; 2] {
    [
        Invocation::Builtin(Builtin::RebuildInitramfs {
            root: InitramfsRoot::Repair,
        }),
        run(inside(Program::GrubMkconfig).args(["-o".to_owned(), format!("/{GRUB_CONFIG}")])),
    ]
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
