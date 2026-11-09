use std::path::{Path, PathBuf};

use toolbox_plan::Editor;

use super::{editor_path, lookup, path};
use crate::Environment;

fn only(existing: &'static [&'static str]) -> impl Fn(&Path) -> bool {
    move |candidate| existing.iter().any(|path| Path::new(path) == candidate)
}

#[test]
fn a_release_runs_the_fixed_path_even_when_it_is_missing() {
    let found = path(
        "/usr/bin/systemctl",
        false,
        only(&["/run/current-system/sw/bin/systemctl"]),
    );

    assert_eq!(found, PathBuf::from("/usr/bin/systemctl"));
}

#[test]
fn a_development_build_keeps_a_fixed_path_that_exists() {
    let found = path(
        "/usr/bin/systemctl",
        true,
        only(&["/usr/bin/systemctl", "/run/current-system/sw/bin/systemctl"]),
    );

    assert_eq!(found, PathBuf::from("/usr/bin/systemctl"));
}

#[test]
fn a_development_build_on_nixos_finds_the_system_profile() {
    let found = path(
        "/usr/bin/systemctl",
        true,
        only(&["/run/current-system/sw/bin/systemctl"]),
    );

    assert_eq!(found, PathBuf::from("/run/current-system/sw/bin/systemctl"));
}

#[test]
fn a_setuid_program_comes_from_the_wrappers() {
    let found = path("/usr/bin/chsh", true, only(&["/run/wrappers/bin/chsh"]));

    assert_eq!(found, PathBuf::from("/run/wrappers/bin/chsh"));
}

#[test]
fn a_program_found_nowhere_keeps_its_fixed_path_for_the_error() {
    let found = path("/usr/bin/garuda-update", true, only(&[]));

    assert_eq!(found, PathBuf::from("/usr/bin/garuda-update"));
}

#[test]
fn an_editor_at_its_fixed_path_runs_from_there() {
    let found = editor_path(
        "/usr/bin/micro",
        Some("/home/me/.local/bin:/run/current-system/sw/bin".as_ref()),
        only(&["/usr/bin/micro", "/home/me/.local/bin/micro"]),
    );

    assert_eq!(found, Some(PathBuf::from("/usr/bin/micro")));
}

#[test]
fn a_missing_editor_is_looked_up_in_the_users_path() {
    let found = editor_path(
        "/usr/bin/micro",
        Some("relative/bin:/home/me/.local/bin:/run/current-system/sw/bin".as_ref()),
        only(&["relative/bin/micro", "/run/current-system/sw/bin/micro"]),
    );

    // A relative folder of PATH would depend on where the app started.
    assert_eq!(
        found,
        Some(PathBuf::from("/run/current-system/sw/bin/micro"))
    );
}

#[test]
fn an_editor_found_nowhere_is_none() {
    assert_eq!(
        editor_path("/usr/bin/micro", Some("/bin".as_ref()), only(&[])),
        None
    );
    assert_eq!(editor_path("/usr/bin/micro", None, only(&[])), None);
}

#[test]
fn a_root_step_never_looks_for_an_editor_in_the_callers_path() {
    let found = lookup(
        Editor::Micro.program(),
        Environment::Clean,
        Some("/home/me/.local/bin".as_ref()),
        false,
        only(&["/home/me/.local/bin/micro"]),
    );

    assert_eq!(found, PathBuf::from("/usr/bin/micro"));
}

#[test]
fn a_user_step_finds_a_missing_editor_in_the_users_path() {
    let found = lookup(
        Editor::Micro.program(),
        Environment::Inherited,
        Some("/home/me/.local/bin".as_ref()),
        false,
        only(&["/home/me/.local/bin/micro"]),
    );

    assert_eq!(found, PathBuf::from("/home/me/.local/bin/micro"));
}
