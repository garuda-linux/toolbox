mod secrets;

use crate::action::testing::shown;
use crate::procedure::{Command, Invocation, Program};
use crate::{
    BootloaderId, DiskDevice, InstalledSystem, PartitionDevice, Password, RepairRoot, RepairWork,
    RootAction, RootLayout, SecretInput, VolumeDevice,
};

const PASSPHRASE: &str = "correct horse: battery staple";
const ALWAYS: &str = "  # always, even after a failure";
const FED: &str = "  # the secret arrives on its standard input";
const ROOT: &str = "/run/garuda-toolbox/boot-repair/root";
const RECHECK: &str = "check the devices of the repair again";

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
        shown(&action),
        [
            format!("mkdir -p {ROOT}  # for root alone, nothing may be mounted there"),
            format!("mount -o subvol=@ /dev/nvme0n1p2 {ROOT}"),
            format!(
                "check that {ROOT} holds an installed system with a kernel, grub-install, grub-mkconfig and the folder /boot/efi"
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
            format!("chroot {ROOT} /usr/bin/grub-mkconfig -o /boot/grub/grub.cfg"),
            format!("umount -R {ROOT}/boot/efi{ALWAYS}"),
            format!("umount -R {ROOT}/sys/firmware/efi/efivars{ALWAYS}"),
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
        shown(&action),
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
    let action = repair(
        locked("/dev/sda2", RootLayout::Subvolume),
        RepairWork::RegenerateConfig,
    );

    assert_eq!(
        shown(&action),
        [
            format!("mkdir -p {ROOT}  # for root alone, nothing may be mounted there"),
            format!(
                "cryptsetup open --type luks --key-file - /dev/sda2 garuda-toolbox-repair{FED}"
            ),
            format!("mount -o subvol=@ /dev/mapper/garuda-toolbox-repair {ROOT}"),
            format!("check that {ROOT} holds an installed system with a kernel, grub-mkconfig"),
            format!("mount --bind /dev {ROOT}/dev"),
            format!("mount --bind /proc {ROOT}/proc"),
            format!("mount --bind /sys {ROOT}/sys"),
            format!("mount --bind /run {ROOT}/run"),
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
            for work in [
                RepairWork::RegenerateConfig,
                uefi("/dev/sda1"),
                bios("/dev/sda"),
            ] {
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

#[test]
fn every_mount_is_undone_by_an_always_step_in_reverse_order() {
    for action in every_repair() {
        let procedure = action.procedure();
        let mut expected = places(&procedure, Program::Mount, run_command);

        expected.reverse();
        assert_eq!(expected.last().map(String::as_str), Some(ROOT));
        assert_eq!(
            places(&procedure, Program::Umount, always_command),
            expected,
            "{action:?}"
        );
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
/// and inside the chroot only GRUB's two. No `dd`, no shell.
#[test]
fn only_the_listed_programs_run() {
    for action in every_repair() {
        for invocation in action.procedure() {
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
                    Some("/usr/bin/grub-install" | "/usr/bin/grub-mkconfig")
                ));

            assert!(allowed, "{invocation}");
        }
    }
}
