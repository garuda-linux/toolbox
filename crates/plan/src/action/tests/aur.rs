use super::packages;
use crate::action::testing::{shown, shown_user};
use crate::action::{RootAction, UserAction};
use crate::procedure::{Builtin, Command, Invocation, Program};
use crate::{BuildRun, InstallReason, PackageName};

fn run() -> BuildRun {
    BuildRun::new("1a2b").unwrap()
}

#[test]
fn repo_dependencies_install_as_dependencies() {
    let action = RootAction::InstallDependencies {
        packages: packages(&["qt6-tools", "cmake"]),
    };

    assert_eq!(
        action.procedure(),
        [Invocation::RunWithUnmetFrom {
            test: Command::new(Program::Pacman).args(["-T", "--", "cmake", "qt6-tools"]),
            run: Command::new(Program::Pacman).args([
                "-S",
                "--needed",
                "--asdeps",
                "--noconfirm",
                "--"
            ]),
        }]
    );
    assert_eq!(
        shown(&action),
        [
            "pacman -S --needed --asdeps --noconfirm -- $(pacman -T -- cmake qt6-tools)  # only the packages pacman -T names as missing, so an installed one keeps its install reason"
        ]
    );
}

/// The request carries the run and the names, never a path.
#[test]
fn built_packages_install_through_the_helper_from_the_run() {
    let action = RootAction::InstallBuilt {
        run: run(),
        packages: packages(&["octopi-git"]),
        reason: InstallReason::Explicit,
    };

    assert_eq!(
        action.procedure(),
        [Invocation::Builtin(Builtin::InstallBuilt {
            run: run(),
            packages: packages(&["octopi-git"]),
            reason: InstallReason::Explicit,
        })]
    );
    assert_eq!(
        shown(&action),
        [
            "copy octopi-git from ~/.cache/garuda-toolbox/aur-built/1a2b to /var/cache/garuda-toolbox/aur, then pacman -U --asexplicit --noconfirm --"
        ]
    );
}

fn built_with(reason: InstallReason) -> String {
    let action = RootAction::InstallBuilt {
        run: run(),
        packages: packages(&["alpm_octopi_utils-git"]),
        reason,
    };

    shown(&action).remove(0)
}

#[test]
fn a_built_dependency_is_marked_as_one() {
    assert!(built_with(InstallReason::Dependency).ends_with("pacman -U --asdeps --noconfirm --"));
}

#[test]
fn an_upgrade_keeps_the_reason_pacman_has() {
    assert!(built_with(InstallReason::Keep).ends_with(", then pacman -U --noconfirm --"));
}

#[test]
fn each_reason_has_its_flag() {
    assert_eq!(InstallReason::Explicit.pacman_flag(), Some("--asexplicit"));
    assert_eq!(InstallReason::Dependency.pacman_flag(), Some("--asdeps"));
    assert_eq!(InstallReason::Keep.pacman_flag(), None);
}

#[test]
fn a_build_runs_makepkg_as_the_user_into_its_run() {
    let action = UserAction::BuildAur {
        base: PackageName::new("octopi-git").unwrap(),
        run: run(),
    };

    assert_eq!(
        action.procedure(),
        [Invocation::Builtin(Builtin::BuildAur {
            base: PackageName::new("octopi-git").unwrap(),
            run: run(),
        })]
    );
    assert_eq!(
        shown_user(&action),
        [
            "PKGDEST=~/.cache/garuda-toolbox/aur-built/1a2b makepkg, in ~/.cache/paru/clone/octopi-git"
        ]
    );
}
