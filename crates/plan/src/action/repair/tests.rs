mod sdboot;
mod secrets;

use crate::procedure::{Builtin, Command, InitramfsRoot, Invocation, Program};
use crate::{
    BootloaderId, DiskDevice, InstalledLoader, InstalledSystem, PartitionDevice, Password,
    RepairRoot, RepairWork, RootAction, RootLayout, SecretInput, VolumeDevice, boot_loader_steps,
};

const PASSPHRASE: &str = "correct horse: battery staple";
const ALWAYS: &str = "  # always, even after a failure";
const FED: &str = "  # the secret arrives on its standard input";
const ROOT: &str = "/run/garuda-toolbox/boot-repair/root";
const RECHECK: &str = "check the devices of the repair again";
const REBUILD: &str = "chroot /run/garuda-toolbox/boot-repair/root /usr/bin/dracut-rebuild, or chroot /run/garuda-toolbox/boot-repair/root /usr/bin/mkinitcpio -P where dracut-rebuild is not installed";

fn open(device: &str, layout: RootLayout) -> InstalledSystem {
    InstalledSystem {
        root: RepairRoot::Open {
            device: VolumeDevice::new(device).unwrap(),
        },
        layout,
    }
}

fn locked(partition: &str, layout: RootLayout) -> InstalledSystem {
    InstalledSystem {
        root: RepairRoot::Locked {
            partition: PartitionDevice::new(partition).unwrap(),
        },
        layout,
    }
}

fn uefi(esp: &str) -> RepairWork {
    RepairWork::InstallUefi {
        esp: PartitionDevice::new(esp).unwrap(),
        id: BootloaderId::new("Garuda").unwrap(),
    }
}

fn bios(disk: &str) -> RepairWork {
    RepairWork::InstallBios {
        disk: DiskDevice::new(disk).unwrap(),
    }
}

fn regenerate() -> RepairWork {
    RepairWork::RegenerateConfig { esp: None }
}

fn repair(system: InstalledSystem, work: RepairWork) -> RootAction {
    let passphrase = system
        .is_locked()
        .then(|| Password::new(PASSPHRASE.to_owned()).unwrap());

    RootAction::RepairBoot {
        system,
        work,
        passphrase,
    }
}

#[test]
fn a_uefi_reinstall_on_btrfs_mounts_installs_and_unmounts_in_reverse() {
    let action = repair(
        open("/dev/nvme0n1p2", RootLayout::Subvolume),
        uefi("/dev/nvme0n1p1"),
    );

    assert_eq!(
        lines(&as_run(&action, InstalledLoader::Grub).unwrap()),
        [
            format!("mkdir -p {ROOT}  # for root alone, nothing may be mounted there"),
            format!("mount -o subvol=@ /dev/nvme0n1p2 {ROOT}"),
            format!(
                "check that {ROOT} holds an installed system with a kernel, grub-install, grub-mkconfig and the folder /boot/efi, or on systemd-boot bootctl, kernel-install and its EFI system partition in /etc/fstab"
            ),
            format!("mount --bind /dev {ROOT}/dev"),
            format!("mount --bind /proc {ROOT}/proc"),
            format!("mount --bind /sys {ROOT}/sys"),
            format!("mount --bind /run {ROOT}/run"),
            format!("mount -t efivarfs efivarfs {ROOT}/sys/firmware/efi/efivars"),
            format!("mount /dev/nvme0n1p1 {ROOT}/boot/efi"),
            RECHECK.to_owned(),
            format!(
                "chroot {ROOT} /usr/bin/grub-install --target=x86_64-efi --efi-directory=/boot/efi --bootloader-id=Garuda --recheck"
            ),
            REBUILD.to_owned(),
            format!("chroot {ROOT} /usr/bin/grub-mkconfig -o /boot/grub/grub.cfg"),
            format!("umount -R {ROOT}/run{ALWAYS}"),
            format!("umount -R {ROOT}/sys{ALWAYS}"),
            format!("umount -R {ROOT}/proc{ALWAYS}"),
            format!("umount -R {ROOT}/dev{ALWAYS}"),
            format!("umount -R {ROOT}{ALWAYS}"),
        ]
    );
}

#[test]
fn a_bios_reinstall_writes_to_the_whole_disk_and_mounts_no_efi_parts() {
    let action = repair(open("/dev/sda2", RootLayout::Plain), bios("/dev/sda"));

    assert_eq!(
        lines(&as_run(&action, InstalledLoader::Grub).unwrap()),
        [
            format!("mkdir -p {ROOT}  # for root alone, nothing may be mounted there"),
            format!("mount /dev/sda2 {ROOT}"),
            format!(
                "check that {ROOT} holds an installed system with a kernel, grub-install and grub-mkconfig"
            ),
            format!("mount --bind /dev {ROOT}/dev"),
            format!("mount --bind /proc {ROOT}/proc"),
            format!("mount --bind /sys {ROOT}/sys"),
            format!("mount --bind /run {ROOT}/run"),
            RECHECK.to_owned(),
            format!("chroot {ROOT} /usr/bin/grub-install --target=i386-pc --recheck /dev/sda"),
            REBUILD.to_owned(),
            format!("chroot {ROOT} /usr/bin/grub-mkconfig -o /boot/grub/grub.cfg"),
            format!("umount -R {ROOT}/run{ALWAYS}"),
            format!("umount -R {ROOT}/sys{ALWAYS}"),
            format!("umount -R {ROOT}/proc{ALWAYS}"),
            format!("umount -R {ROOT}/dev{ALWAYS}"),
            format!("umount -R {ROOT}{ALWAYS}"),
        ]
    );
}

#[test]
fn regenerating_the_menu_of_a_locked_system_unlocks_and_closes_it() {
    let action = repair(locked("/dev/sda2", RootLayout::Subvolume), regenerate());

    assert_eq!(
        lines(&as_run(&action, InstalledLoader::Grub).unwrap()),
        [
            format!("mkdir -p {ROOT}  # for root alone, nothing may be mounted there"),
            format!(
                "cryptsetup open --type luks --key-file - /dev/sda2 garuda-toolbox-repair{FED}"
            ),
            format!("mount -o subvol=@ /dev/mapper/garuda-toolbox-repair {ROOT}"),
            format!(
                "check that {ROOT} holds an installed system with a kernel, grub-mkconfig, or on systemd-boot kernel-install"
            ),
            format!("mount --bind /dev {ROOT}/dev"),
            format!("mount --bind /proc {ROOT}/proc"),
            format!("mount --bind /sys {ROOT}/sys"),
            format!("mount --bind /run {ROOT}/run"),
            REBUILD.to_owned(),
            format!("chroot {ROOT} /usr/bin/grub-mkconfig -o /boot/grub/grub.cfg"),
            format!("umount -R {ROOT}/run{ALWAYS}"),
            format!("umount -R {ROOT}/sys{ALWAYS}"),
            format!("umount -R {ROOT}/proc{ALWAYS}"),
            format!("umount -R {ROOT}/dev{ALWAYS}"),
            format!("umount -R {ROOT}{ALWAYS}"),
            format!("cryptsetup close garuda-toolbox-repair{ALWAYS}"),
        ]
    );
}

fn every_repair() -> Vec<RootAction> {
    let mut all = Vec::new();

    for layout in [RootLayout::Plain, RootLayout::Subvolume] {
        for system in [
            open("/dev/sda2", layout),
            open("/dev/mapper/vg0-root", layout),
            locked("/dev/nvme0n1p3", layout),
        ] {
            for work in [regenerate(), uefi("/dev/sda1"), bios("/dev/sda")] {
                all.push(repair(system.clone(), work));
            }
        }
    }

    all
}

/// The last argument of every command of `program` that `pick` takes
/// out of an invocation: the mount points, in order.
fn places(
    procedure: &[Invocation],
    program: Program,
    pick: fn(&Invocation) -> Option<&Command>,
) -> Vec<String> {
    procedure
        .iter()
        .filter_map(pick)
        .filter(|command| command.program() == program)
        .filter_map(|command| command.arguments().last().cloned())
        .collect()
}

fn run_command(invocation: &Invocation) -> Option<&Command> {
    if let Invocation::Run(command) = invocation {
        Some(command)
    } else {
        None
    }
}

fn always_command(invocation: &Invocation) -> Option<&Command> {
    if let Invocation::Always(command) = invocation {
        Some(command)
    } else {
        None
    }
}

fn fed(invocation: &Invocation) -> Option<&SecretInput> {
    if let Invocation::Feed { command: _, input } = invocation {
        input.as_ref()
    } else {
        None
    }
}

/// Every repair as the helper runs it on each boot loader, where it fits.
fn every_run() -> Vec<(RootAction, Vec<Invocation>)> {
    let loaders = [
        InstalledLoader::Grub,
        InstalledLoader::SystemdBoot {
            esp: Some(crate::EspPlace::Efi),
        },
    ];

    every_repair()
        .into_iter()
        .flat_map(|action| {
            loaders
                .into_iter()
                .filter_map(|loader| as_run(&action, loader).ok())
                .map(|steps| (action.clone(), steps))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The root and the binds are undone in reverse order. The boot loader's
/// own mounts lie below them, so their `umount -R` takes those along.
#[test]
fn every_mount_is_undone_by_an_always_step() {
    for (action, steps) in every_run() {
        let unmounted = places(&steps, Program::Umount, always_command);
        let mut own = places(&action.procedure(), Program::Mount, run_command);

        own.reverse();
        assert_eq!(unmounted, own, "{action:?}");
        assert_eq!(unmounted.last().map(String::as_str), Some(ROOT));

        for point in places(&steps, Program::Mount, run_command) {
            assert!(
                unmounted
                    .iter()
                    .any(|undone| point == *undone || point.starts_with(&format!("{undone}/"))),
                "{point} in {action:?}"
            );
        }
    }
}

#[test]
fn an_unlock_is_closed_by_the_last_always_step_and_only_then() {
    for action in every_repair() {
        let procedure = action.procedure();
        let opens = procedure
            .iter()
            .filter(|invocation| {
                matches!(invocation, Invocation::Feed { command, input: _ }
                    if command.program() == Program::Cryptsetup)
            })
            .count();

        let closes_last = matches!(procedure.last(), Some(Invocation::Always(command))
            if command.program() == Program::Cryptsetup
                && command.arguments() == ["close", "garuda-toolbox-repair"]);

        assert_eq!(opens, usize::from(action.needs_secret()), "{action:?}");
        assert_eq!(closes_last, action.needs_secret(), "{action:?}");
    }
}

/// After the first cleanup step nothing else may follow: a `Run` there
/// would be skipped after a failure, and the cleanup must be the end.
#[test]
fn nothing_but_cleanup_follows_the_first_always_step() {
    for action in every_repair() {
        let procedure = action.procedure();
        let first = procedure
            .iter()
            .position(|invocation| matches!(invocation, Invocation::Always(_)))
            .unwrap();

        assert!(
            procedure
                .iter()
                .skip(first)
                .all(|invocation| matches!(invocation, Invocation::Always(_))),
            "{action:?}"
        );
    }
}

/// `mount`, `umount`, `cryptsetup` and `chroot` are the only programs,
/// and inside the chroot only the boot loaders' installers and GRUB's
/// menu builder. No `dd`, no shell.
#[test]
fn only_the_listed_programs_run() {
    for (_, steps) in every_run() {
        for invocation in steps {
            let command = match &invocation {
                Invocation::Run(command)
                | Invocation::Always(command)
                | Invocation::Feed { command, input: _ } => command,
                Invocation::Builtin(_) => continue,
                Invocation::RunWithTargetsFrom { query: _, run: _ } => {
                    panic!("a repair queries nothing")
                }
            };

            let program = command.program();
            let inside = command.arguments().get(1).map(String::as_str);
            let allowed = matches!(
                program,
                Program::Mount | Program::Umount | Program::Cryptsetup
            ) || (program == Program::Chroot
                && matches!(
                    inside,
                    Some("/usr/bin/grub-install" | "/usr/bin/grub-mkconfig" | "/usr/bin/bootctl")
                ));

            assert!(allowed, "{invocation}");
        }
    }
}

/// `grub-mkconfig` writes an initrd line only for an initramfs it finds.
#[test]
fn every_repair_builds_the_initramfs_right_before_the_menu() {
    for action in every_repair() {
        let procedure = as_run(&action, InstalledLoader::Grub).unwrap();
        let menu = procedure
            .iter()
            .position(|invocation| {
                run_command(invocation).is_some_and(|command| {
                    command.arguments().get(1).map(String::as_str) == Some("/usr/bin/grub-mkconfig")
                })
            })
            .unwrap();

        assert_eq!(
            procedure.get(menu.wrapping_sub(1)),
            Some(&Invocation::Builtin(Builtin::RebuildInitramfs {
                root: InitramfsRoot::Repair
            })),
            "{action:?}"
        );
    }
}

/// What the helper runs once it has seen `loader`: the
/// [`Builtin::RepairBootLoader`] step replaced by its steps.
fn as_run(action: &RootAction, loader: InstalledLoader) -> Result<Vec<Invocation>, &'static str> {
    let RootAction::RepairBoot {
        system,
        work,
        passphrase: _,
    } = action
    else {
        panic!("no repair: {action:?}");
    };

    let mut steps = Vec::new();

    for invocation in action.procedure() {
        if matches!(
            invocation,
            Invocation::Builtin(Builtin::RepairBootLoader { system: _, work: _ })
        ) {
            steps.extend(boot_loader_steps(system, work, loader)?);
        } else {
            steps.push(invocation);
        }
    }

    Ok(steps)
}

fn lines(steps: &[Invocation]) -> Vec<String> {
    steps.iter().map(ToString::to_string).collect()
}
