use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::fs::symlink;

use tempfile::TempDir;
use toolbox_plan::FetchRefusal;

use super::{RealPath, Unresolved, resolve};
use crate::config_file::tests::path;

/// A root folder with `etc/real.conf`, `etc/conf.d/a.conf` and the folders
/// links like to lead to.
fn system() -> TempDir {
    let root = tempfile::tempdir().unwrap();

    for folder in ["etc/conf.d", "usr/lib", "run/systemd", "nix/store", "home"] {
        fs::create_dir_all(root.path().join(folder)).unwrap();
    }

    fs::write(root.path().join("etc/real.conf"), "a = 1\n").unwrap();
    fs::write(root.path().join("etc/conf.d/a.conf"), "a = 1\n").unwrap();
    root
}

fn real(root: &TempDir, text: &str) -> RealPath {
    match resolve(root.path(), &path(text)) {
        Ok(real) => real,
        Err(error) => panic!("{text} did not resolve: {error:?}"),
    }
}

fn refused(root: &TempDir, text: &str) -> (FetchRefusal, String) {
    match resolve(root.path(), &path(text)) {
        Err(Unresolved::Refused { reason, target }) => (reason, target),
        other => panic!("{text} was not refused: {other:?}"),
    }
}

#[test]
fn a_path_without_a_link_is_its_own_real_path() {
    let root = system();
    let found = real(&root, "/etc/conf.d/a.conf");

    assert_eq!(found.path, path("/etc/conf.d/a.conf"));
    assert!(!found.linked);
    assert!(matches!(
        resolve(root.path(), &path("/etc/nothing")),
        Err(Unresolved::Missing)
    ));
    assert!(matches!(
        resolve(root.path(), &path("/etc/none/file")),
        Err(Unresolved::Missing)
    ));
    assert!(matches!(
        resolve(root.path(), &path("/etc/real.conf/below-a-file")),
        Err(Unresolved::Missing)
    ));
}

#[test]
fn relative_and_absolute_links_lead_to_their_target_below_the_root() {
    let root = system();

    symlink("real.conf", root.path().join("etc/beside.conf")).unwrap();
    symlink(
        "../etc/conf.d/a.conf",
        root.path().join("etc/climbing.conf"),
    )
    .unwrap();
    symlink("/etc/real.conf", root.path().join("etc/absolute.conf")).unwrap();
    symlink("../../../etc/real.conf", root.path().join("etc/above.conf")).unwrap();

    for (link, target) in [
        ("/etc/beside.conf", "/etc/real.conf"),
        ("/etc/climbing.conf", "/etc/conf.d/a.conf"),
        ("/etc/absolute.conf", "/etc/real.conf"),
        ("/etc/above.conf", "/etc/real.conf"),
    ] {
        let found = real(&root, link);

        assert_eq!(found.path, path(target), "{link}");
        assert!(found.linked, "{link}");
    }
}

#[test]
fn a_chain_of_links_and_a_linked_folder_are_followed_to_the_end() {
    let root = system();

    symlink("second.conf", root.path().join("etc/first.conf")).unwrap();
    symlink("/etc/linked.d/a.conf", root.path().join("etc/second.conf")).unwrap();
    symlink("conf.d", root.path().join("etc/linked.d")).unwrap();

    assert_eq!(
        real(&root, "/etc/first.conf").path,
        path("/etc/conf.d/a.conf")
    );
    assert_eq!(
        real(&root, "/etc/linked.d/a.conf").path,
        path("/etc/conf.d/a.conf")
    );
    assert!(real(&root, "/etc/linked.d").linked);
    assert!(matches!(
        resolve(root.path(), &path("/etc/linked.d/none.conf")),
        Err(Unresolved::Missing)
    ));
}

#[test]
fn a_link_to_nothing_or_in_a_circle_is_dangling() {
    let root = system();

    symlink("gone.conf", root.path().join("etc/dangling.conf")).unwrap();
    symlink("/etc/none/gone.conf", root.path().join("etc/deep.conf")).unwrap();
    symlink("b.conf", root.path().join("etc/a.conf")).unwrap();
    symlink("a.conf", root.path().join("etc/b.conf")).unwrap();
    symlink("self.conf", root.path().join("etc/self.conf")).unwrap();

    assert_eq!(
        refused(&root, "/etc/dangling.conf"),
        (FetchRefusal::Dangling, "/etc/gone.conf".to_owned())
    );
    assert_eq!(
        refused(&root, "/etc/deep.conf"),
        (FetchRefusal::Dangling, "/etc/none/gone.conf".to_owned())
    );
    assert_eq!(refused(&root, "/etc/a.conf").0, FetchRefusal::Dangling);
    assert_eq!(refused(&root, "/etc/self.conf").0, FetchRefusal::Dangling);
}

#[test]
fn a_link_out_of_the_config_folders_is_refused_with_its_target() {
    let root = system();

    for (name, target) in [
        ("os-release", "../usr/lib/os-release"),
        ("resolv.conf", "/run/systemd/stub-resolv.conf"),
        ("fstab", "/nix/store/abc-fstab"),
        ("dotfile.conf", "/home/dotfile.conf"),
    ] {
        let below_root = target.trim_start_matches("../").trim_start_matches('/');

        fs::write(root.path().join(below_root), "x\n").unwrap();
        symlink(target, root.path().join("etc").join(name)).unwrap();

        assert_eq!(
            refused(&root, &format!("/etc/{name}")),
            (FetchRefusal::Outside, format!("/{below_root}")),
            "{name}"
        );
    }

    // Folders that need not exist in a test root: the walk ends where the
    // path does, so these dangle, and still nothing is handed out.
    for (name, target) in [
        ("mtab", "../proc/self/mounts"),
        ("kernel", "/sys/kernel/notes"),
        ("null", "/dev/null"),
    ] {
        symlink(target, root.path().join("etc").join(name)).unwrap();
        assert_eq!(
            refused(&root, &format!("/etc/{name}")).0,
            FetchRefusal::Dangling,
            "{name}"
        );
    }
}

/// A link in a folder another user can write would let that user choose
/// which file an administrator edits, so a chain that leaves the config
/// folders ends there even when it comes back.
#[test]
fn a_chain_that_leaves_the_config_folders_is_refused_even_when_it_returns() {
    let root = system();

    symlink("/home/hop.conf", root.path().join("etc/detour.conf")).unwrap();
    symlink("/etc/real.conf", root.path().join("home/hop.conf")).unwrap();
    symlink("/home", root.path().join("etc/away")).unwrap();
    symlink("/etc/conf.d", root.path().join("home/back")).unwrap();

    assert_eq!(
        refused(&root, "/etc/detour.conf"),
        (FetchRefusal::Outside, "/home/hop.conf".to_owned())
    );
    assert_eq!(
        refused(&root, "/etc/away/back/a.conf"),
        (FetchRefusal::Outside, "/home/back/a.conf".to_owned())
    );

    // Dots that only pass through another folder follow no link there.
    symlink(
        "../home/../etc/real.conf",
        root.path().join("etc/dots.conf"),
    )
    .unwrap();
    assert_eq!(real(&root, "/etc/dots.conf").path, path("/etc/real.conf"));
}

#[test]
fn a_link_to_the_password_hashes_is_refused() {
    let root = system();

    fs::write(root.path().join("etc/shadow"), "root:$6$hash\n").unwrap();
    symlink("shadow", root.path().join("etc/innocent.conf")).unwrap();
    symlink("/etc/innocent.conf", root.path().join("etc/twice.conf")).unwrap();

    assert_eq!(
        refused(&root, "/etc/innocent.conf"),
        (FetchRefusal::Outside, "/etc/shadow".to_owned())
    );
    assert_eq!(refused(&root, "/etc/twice.conf").0, FetchRefusal::Outside);
}

#[test]
fn a_target_with_control_characters_is_refused_and_shown_without_them() {
    let root = system();

    symlink("line\nbreak\u{1b}[31m", root.path().join("etc/odd.conf")).unwrap();
    fs::write(root.path().join("etc/line\nbreak\u{1b}[31m"), "x\n").unwrap();

    let (reason, target) = refused(&root, "/etc/odd.conf");

    assert_eq!(reason, FetchRefusal::Outside);
    assert!(!target.chars().any(char::is_control), "{target:?}");
    assert!(target.starts_with("/etc/line"), "{target:?}");
}

#[test]
fn a_link_with_no_readable_target_is_outside() {
    let root = system();

    symlink(
        OsStr::from_bytes(b"/etc/\xff.conf"),
        root.path().join("etc/broken.conf"),
    )
    .unwrap();

    assert_eq!(refused(&root, "/etc/broken.conf").0, FetchRefusal::Outside);
}
