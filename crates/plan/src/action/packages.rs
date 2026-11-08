use super::{RemoteAction, TargetList, Upgrader, names};
use crate::PackageName;
use crate::procedure::{Command, Invocation, Program};

pub(super) fn install(packages: &TargetList<PackageName>) -> Command {
    pacman(["-S", "--needed", "--noconfirm", "--"]).args(names(packages))
}

pub(super) fn remove_orphans() -> Invocation {
    Invocation::RunWithTargetsFrom {
        query: pacman(["-Qtdq"]),
        run: remove(),
    }
}

pub(super) fn remove() -> Command {
    pacman(["-Rns", "--noconfirm", "--"])
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
