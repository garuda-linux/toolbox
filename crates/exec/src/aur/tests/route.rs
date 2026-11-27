use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::{PermissionsExt as _, symlink};
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use toolbox_plan::testing::accounts::PASSWD;
use toolbox_plan::{Caller, InstallReason};

use super::{HOME, octopi, passwd_with_caller, system, uid};
use crate::aur::route::{RouteProblem, route};
use crate::aur::{StageError, install};

const KINDS: [&str; 7] = [
    "NoAccount",
    "Passwd",
    "Home",
    "Cache",
    "Folder",
    "Foreign",
    "Shared",
];

/// One refused setup. `other_user` makes the caller the owner's next uid.
struct Case {
    kind: &'static str,
    other_user: bool,
    setup: fn(&Path),
}

fn kind(problem: &RouteProblem) -> &'static str {
    match problem {
        RouteProblem::NoAccount(_) => "NoAccount",
        RouteProblem::Passwd(_) => "Passwd",
        RouteProblem::Home(_) => "Home",
        RouteProblem::Cache(_) => "Cache",
        RouteProblem::Folder { name: _, error: _ } => "Folder",
        RouteProblem::Foreign(_) => "Foreign",
        RouteProblem::Shared(_) => "Shared",
    }
}

fn home(root: &Path) -> PathBuf {
    root.join(HOME)
}

fn cache(root: &Path) -> PathBuf {
    home(root).join(".cache")
}

fn elsewhere(root: &Path) -> PathBuf {
    let folder = root.join("elsewhere");
    fs::create_dir_all(&folder).unwrap();
    folder
}

fn cases() -> Vec<Case> {
    let mut all = early_cases();
    all.extend(late_cases());
    all
}

fn early_cases() -> Vec<Case> {
    vec![
        Case {
            kind: "NoAccount",
            other_user: false,
            setup: |root| {
                let passwd = PASSWD.replace("user:x:1000:1000:", "user:x:4242:1000:");
                fs::write(root.join("etc/passwd"), passwd).unwrap();
            },
        },
        Case {
            kind: "Passwd",
            other_user: false,
            setup: |root| fs::remove_file(root.join("etc/passwd")).unwrap(),
        },
        Case {
            kind: "Home",
            other_user: false,
            setup: |root| fs::remove_dir_all(home(root)).unwrap(),
        },
        Case {
            kind: "Home",
            other_user: false,
            setup: |root| {
                fs::remove_dir_all(home(root)).unwrap();
                symlink(elsewhere(root), home(root)).unwrap();
            },
        },
        Case {
            kind: "Cache",
            other_user: false,
            setup: |root| fs::remove_dir_all(cache(root)).unwrap(),
        },
        Case {
            kind: "Cache",
            other_user: false,
            setup: |root| {
                fs::remove_dir_all(cache(root)).unwrap();
                symlink(elsewhere(root), cache(root)).unwrap();
            },
        },
        Case {
            kind: "Shared",
            other_user: false,
            setup: |root| {
                fs::set_permissions(cache(root), fs::Permissions::from_mode(0o775)).unwrap();
            },
        },
    ]
}

fn late_cases() -> Vec<Case> {
    vec![
        Case {
            kind: "Folder",
            other_user: false,
            setup: |root| {
                let folder = cache(root).join("garuda-toolbox");
                fs::remove_dir_all(&folder).unwrap();
                symlink(elsewhere(root), &folder).unwrap();
            },
        },
        Case {
            kind: "Folder",
            other_user: false,
            setup: |root| {
                let folder = cache(root).join("garuda-toolbox/aur-built");
                fs::remove_dir_all(&folder).unwrap();
                symlink(elsewhere(root), &folder).unwrap();
            },
        },
        Case {
            kind: "Foreign",
            other_user: true,
            setup: |_root| {},
        },
    ]
}

/// A root with the caller in passwd and the setup applied. The caller is
/// the next uid for `other_user`, so the folders of `uid()` are not its own.
fn refused(case: &Case) -> (TempDir, u32) {
    let (root, _folder) = system();
    passwd_with_caller(root.path());
    let caller = if case.other_user { uid() + 1 } else { uid() };

    if case.other_user {
        let passwd = root.path().join("etc/passwd");
        let text = fs::read_to_string(&passwd).unwrap().replace(
            &format!("user:x:{}:1000:", uid()),
            &format!("user:x:{caller}:1000:"),
        );

        fs::write(passwd, text).unwrap();
    }

    (case.setup)(root.path());
    (root, caller)
}

#[test]
fn a_plain_home_with_a_cache_passes() {
    let (root, _folder) = system();
    passwd_with_caller(root.path());

    let found = route(root.path(), uid()).unwrap();

    assert_eq!(found.missing, Vec::<&str>::new());
}

#[test]
fn a_home_without_the_garuda_folders_passes() {
    let (root, _folder) = system();
    passwd_with_caller(root.path());
    fs::remove_dir_all(cache(root.path()).join("garuda-toolbox")).unwrap();

    let found = route(root.path(), uid()).unwrap();

    assert_eq!(found.missing, ["garuda-toolbox", "aur-built"]);
}

#[test]
fn each_refused_setup_gives_its_own_problem() {
    for case in cases() {
        let (root, caller) = refused(&case);

        let problem = route(root.path(), caller).unwrap_err();

        assert_eq!(kind(&problem), case.kind, "{problem}");
    }
}

/// The helper's own path: `install` looks the home up and stages it.
#[test]
fn the_helper_refuses_what_the_check_refuses_with_the_same_message() {
    for case in cases() {
        let (root, caller) = refused(&case);
        let problem = route(root.path(), caller).unwrap_err();

        let stage = install(
            root.path(),
            Caller::new(caller),
            octopi(InstallReason::Explicit),
            |_command| panic!("pacman ran on a refused route"),
        )
        .unwrap_err();

        assert_eq!(
            StageError::from(problem).to_string(),
            stage.to_string(),
            "{}",
            case.kind
        );
    }
}

#[test]
fn every_problem_has_a_setup_that_produces_it() {
    let mut produced = BTreeSet::new();
    for case in cases() {
        let (root, caller) = refused(&case);
        produced.insert(kind(&route(root.path(), caller).unwrap_err()));
    }

    assert_eq!(produced, KINDS.into_iter().collect());
}
