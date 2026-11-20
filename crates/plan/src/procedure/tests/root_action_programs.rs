use super::root_actions::every_root_action;
use crate::procedure::{Builtin, Command, Invocation, Program, initramfs_rebuilds, uki_rebuilds};
use crate::{Editor, EspPlace, InstalledLoader, boot_loader_steps};

/// The programs a root action must never run: the AUR helper and makepkg run as the
/// user, and the editors belong to user steps only.
fn never_as_root(program: Program) -> bool {
    matches!(program, Program::Paru | Program::Makepkg)
        || Editor::ALL.iter().any(|editor| editor.program() == program)
}

/// The programs of one step. A builtin that runs programs of its own is
/// unfolded to the programs it may run.
fn programs(invocation: &Invocation) -> Vec<Program> {
    match invocation {
        Invocation::Run(command)
        | Invocation::Always(command)
        | Invocation::Cleanup {
            command,
            leftover: _,
        }
        | Invocation::Feed { command, input: _ } => vec![command.program()],
        Invocation::RunWithUnmetFrom {
            test: first,
            run: second,
        }
        | Invocation::RunWithTargetsFrom {
            query: first,
            run: second,
        }
        | Invocation::RunUnlessItRemovesProtected {
            preview: first,
            run: second,
        } => vec![first.program(), second.program()],
        Invocation::Builtin(builtin) => builtin_programs(builtin),
    }
}

fn builtin_programs(builtin: &Builtin) -> Vec<Program> {
    match builtin {
        Builtin::RebuildInitramfs { root: _ } => {
            initramfs_rebuilds().iter().map(Command::program).collect()
        }
        Builtin::RebuildUkis { root: _ } => uki_rebuilds().iter().map(Command::program).collect(),
        Builtin::SnapshotBeforeMerges => vec![crate::merge::snapshot().program()],
        Builtin::InstallBuilt {
            run: _,
            packages: _,
            reason: _,
        } => vec![Program::Pacman],
        Builtin::BuildAur { base: _, run: _ } => vec![Program::Makepkg],
        Builtin::ResolveMerge(merge) => merge.syntax_check().iter().map(Command::program).collect(),
        Builtin::RepairBootLoader { system, work } => loaders()
            .into_iter()
            .flat_map(|loader| boot_loader_steps(system, work, loader).unwrap_or_default())
            .flat_map(|step| programs(&step))
            .collect(),
        // These run no program of their own. Listed without a wildcard, so
        // a new builtin must be decided here.
        Builtin::RemoveDatabaseLock
        | Builtin::InstallMirrorlist(_)
        | Builtin::RequireListedShell(_)
        | Builtin::WriteDnsConfig(_)
        | Builtin::WriteWifiBackend(_)
        | Builtin::SetGarudaUpdateFlag { flag: _, on: _ }
        | Builtin::EditUserConfig { edit: _, on: _ }
        | Builtin::EditLocaleGen { entry: _, on: _ }
        | Builtin::SetSystemLocale(_)
        | Builtin::RequireOrdinaryUser(_)
        | Builtin::RequireFreeHome(_, _)
        | Builtin::RequireSafeHome(_)
        | Builtin::RequireUserGroup(_)
        | Builtin::WriteConfigFile(_)
        | Builtin::WriteHomeFile(_)
        | Builtin::EditModprobeBlacklist { module: _, on: _ }
        | Builtin::EditGrubDefaults(_)
        | Builtin::EditLoaderConf(_)
        | Builtin::WriteKernelCmdline(_)
        | Builtin::PrepareRepairMount
        | Builtin::RequireInstalledSystem(_)
        | Builtin::RecheckRepairDevices { system: _, work: _ } => Vec::new(),
    }
}

/// Every boot loader a repair can find, so the steps of each are walked.
fn loaders() -> Vec<InstalledLoader> {
    vec![
        InstalledLoader::Grub,
        InstalledLoader::SystemdBoot { esp: None },
        InstalledLoader::SystemdBoot {
            esp: Some(EspPlace::Efi),
        },
        InstalledLoader::SystemdBoot {
            esp: Some(EspPlace::Boot),
        },
        InstalledLoader::SystemdBoot {
            esp: Some(EspPlace::BootEfi),
        },
    ]
}

#[test]
fn no_root_action_runs_paru_or_an_editor() {
    let mut walked: usize = 0;

    for action in every_root_action() {
        for invocation in action.procedure() {
            for program in programs(&invocation) {
                walked += 1;
                assert!(
                    !never_as_root(program),
                    "{action:?} runs {}",
                    program.name()
                );
            }
        }
    }

    assert!(walked > 0, "the walk saw no program");
}
