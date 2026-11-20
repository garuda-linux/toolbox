use super::{RemoteAction, TargetList, Upgrader, names};
use crate::procedure::{Builtin, Command, Invocation, Program};
use crate::{BuildRun, InstallReason, PackageName};

pub(super) fn install(packages: &TargetList<PackageName>) -> Command {
    pacman(["-S", "--needed", "--noconfirm", "--"]).args(names(packages))
}

/// Only what is missing, so an installed package named here, even an
/// outdated one, keeps the reason it was installed for.
pub(super) fn install_dependencies(packages: &TargetList<PackageName>) -> Invocation {
    Invocation::RunWithUnmetFrom {
        test: pacman(["-T", "--"]).args(names(packages)),
        run: pacman(["-S", "--needed", "--asdeps", "--noconfirm", "--"]),
    }
}

pub(super) fn install_built(
    run: &BuildRun,
    packages: &TargetList<PackageName>,
    reason: InstallReason,
) -> Invocation {
    Invocation::Builtin(Builtin::InstallBuilt {
        run: run.clone(),
        packages: packages.clone(),
        reason,
    })
}

/// `-Rn` without `-s`, so each round removes only names the helper
/// checked, and a dependency only an orphan needed is checked in the next.
pub(super) fn remove_orphans() -> Invocation {
    Invocation::RunWithTargetsFrom {
        query: pacman(["-Qtdq"]),
        run: pacman(["-Rn", "--noconfirm", "--"]),
    }
}

/// pacman refuses `--print` with `--nosave`, which changes no target.
pub(super) fn remove(packages: &TargetList<PackageName>) -> Invocation {
    Invocation::RunUnlessItRemovesProtected {
        preview: pacman(["-Rs", "--print", "--print-format", "%n", "--"]).args(names(packages)),
        run: pacman(["-Rns", "--noconfirm", "--"]).args(names(packages)),
    }
}

pub(super) fn refresh_keyring() -> Command {
    Command::new(Program::PacmanKey).arg("--refresh-keys")
}

fn garuda_update() -> Command {
    Command::new(Program::GarudaUpdate).env("GARUDA_UPDATE_TOOLBOX", "1")
}

pub(super) fn remote(action: RemoteAction) -> Command {
    garuda_update().args(["remote", action.as_str()])
}

pub(super) fn upgrade(upgrader: Upgrader) -> Command {
    match upgrader {
        Upgrader::GarudaUpdate => garuda_update().arg("--skip-mirrorlist"),
        Upgrader::Pacman => pacman(["-Syu", "--noconfirm"]),
    }
}

fn pacman<const N: usize>(args: [&str; N]) -> Command {
    Command::new(Program::Pacman).args(args)
}
