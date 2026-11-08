//! A symlinked config file is edited as the file it leads to: links are
//! followed once, the target must be a file the toolbox edits, and a link
//! put in the way after that look is never followed.

use std::fmt::Display;
use std::fs;
use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _, symlink};
use std::path::Path;

use tempfile::TempDir;
use toolbox_plan::protocol::{Failure, Outcome};
use toolbox_plan::{ConfigFilePath, ConfigWrite, FetchRefusal, Fingerprint, StepId};

use super::super::write::{write_config_file, write_real};
use super::super::{
    ConfigRead, Found, ReadError, WriteError, execute, inspect, read_config_file, read_real,
    resolve,
};
use super::{BACKUPS, fifo, path, refused, text};
use crate::builtin::FileChange;
use crate::testing::printed;

const OLD: &str = "old = 1\n";
const NEW: &str = "new = 2\n";

/// A root folder with `etc/real/app.conf` (mode 0640) and the link
/// `etc/app.conf` to it, as a package or an administrator would set up.
fn system() -> TempDir {
    let root = tempfile::tempdir().unwrap();
    let real = root.path().join("etc/real/app.conf");

    fs::create_dir_all(root.path().join("etc/real")).unwrap();
    fs::create_dir_all(root.path().join("usr/lib")).unwrap();
    fs::write(&real, OLD).unwrap();
    fs::set_permissions(&real, fs::Permissions::from_mode(0o640)).unwrap();
    symlink("real/app.conf", root.path().join("etc/app.conf")).unwrap();
    root
}

fn is_link(path: &Path) -> bool {
    fs::symlink_metadata(path).unwrap().file_type().is_symlink()
}

fn write(root: &TempDir, file: &str) -> Result<FileChange, WriteError> {
    write_config_file(
        root.path(),
        &path(file),
        &text(NEW),
        Fingerprint::of(OLD.as_bytes()),
    )
}

fn refusal(result: Result<FileChange, WriteError>, what: impl Display) -> FetchRefusal {
    match result {
        Err(WriteError::Refused(reason)) => reason,
        other => panic!("the write of {what} was not refused: {other:?}"),
    }
}

fn write_refused(root: &TempDir, file: &str) -> FetchRefusal {
    refusal(write(root, file), file)
}

fn write_real_refused(root: &TempDir, real: &ConfigFilePath) -> FetchRefusal {
    refusal(
        write_real(
            root.path(),
            real,
            &text(NEW),
            Fingerprint::of(OLD.as_bytes()),
        ),
        real,
    )
}

#[test]
fn a_link_is_read_as_the_file_it_leads_to() {
    let root = system();

    assert_eq!(
        read_config_file(root.path(), &path("/etc/app.conf")).unwrap(),
        ConfigRead {
            path: path("/etc/real/app.conf"),
            content: text(OLD),
        }
    );
    assert_eq!(
        inspect(root.path(), &path("/etc/app.conf")),
        Found::File {
            link: Some(path("/etc/real/app.conf"))
        }
    );
}

#[test]
fn a_write_through_a_link_replaces_the_target_and_keeps_the_link() {
    let root = system();
    let link = root.path().join("etc/app.conf");
    let target = root.path().join("etc/real/app.conf");
    let before = fs::metadata(&target).unwrap();

    assert_eq!(write(&root, "/etc/app.conf").unwrap(), FileChange::Written);

    let after = fs::metadata(&target).unwrap();

    assert!(is_link(&link), "the link was replaced by a file");
    assert_eq!(
        fs::read_link(&link).unwrap(),
        Path::new("real/app.conf"),
        "the link leads elsewhere"
    );
    assert!(!is_link(&target));
    assert_eq!(fs::read_to_string(&target).unwrap(), NEW);
    assert_eq!(after.permissions().mode() & 0o7777, 0o640);
    assert_eq!((after.uid(), after.gid()), (before.uid(), before.gid()));
    assert_ne!(
        after.ino(),
        before.ino(),
        "the target was not replaced whole"
    );
    assert_eq!(
        fs::read_to_string(root.path().join(BACKUPS).join("etc/real/app.conf")).unwrap(),
        OLD,
        "the backup lies under the target's own path"
    );
    assert!(!root.path().join(BACKUPS).join("etc/app.conf").exists());

    let left: Vec<_> = ["etc", "etc/real"]
        .into_iter()
        .flat_map(|folder| fs::read_dir(root.path().join(folder)).unwrap())
        .map(|entry| entry.unwrap().file_name())
        .filter(|name| name.to_string_lossy().contains(".tmp-"))
        .collect();

    assert!(left.is_empty(), "temporary files stayed behind: {left:?}");
}

#[test]
fn a_chain_of_links_ends_at_the_last_target() {
    let root = system();

    symlink("/etc/app.conf", root.path().join("etc/second.conf")).unwrap();
    symlink("second.conf", root.path().join("etc/first.conf")).unwrap();

    assert_eq!(
        read_config_file(root.path(), &path("/etc/first.conf"))
            .unwrap()
            .path,
        path("/etc/real/app.conf")
    );
    assert_eq!(
        write(&root, "/etc/first.conf").unwrap(),
        FileChange::Written
    );
    assert_eq!(
        fs::read_to_string(root.path().join("etc/real/app.conf")).unwrap(),
        NEW
    );

    for link in ["etc/first.conf", "etc/second.conf", "etc/app.conf"] {
        assert!(is_link(&root.path().join(link)), "{link}");
    }
}

#[test]
fn a_dangling_link_is_neither_read_nor_created() {
    let root = system();

    symlink("real/gone.conf", root.path().join("etc/dangling.conf")).unwrap();

    assert_eq!(
        refused(read_config_file(root.path(), &path("/etc/dangling.conf"))),
        FetchRefusal::Dangling
    );
    assert_eq!(
        write_refused(&root, "/etc/dangling.conf"),
        FetchRefusal::Dangling
    );
    assert!(!root.path().join("etc/real/gone.conf").exists());
    assert_eq!(
        inspect(root.path(), &path("/etc/dangling.conf")),
        Found::Link {
            target: "/etc/real/gone.conf".to_owned(),
            reason: FetchRefusal::Dangling
        }
    );
}

#[test]
fn a_link_to_a_folder_or_a_fifo_is_no_file() {
    let root = system();

    symlink("real", root.path().join("etc/folder.conf")).unwrap();
    fifo(&root.path().join("etc/real/pipe"));
    symlink("real/pipe", root.path().join("etc/pipe.conf")).unwrap();

    for link in ["/etc/folder.conf", "/etc/pipe.conf"] {
        assert_eq!(
            refused(read_config_file(root.path(), &path(link))),
            FetchRefusal::NotRegular,
            "{link}"
        );
        assert_eq!(
            write_refused(&root, link),
            FetchRefusal::NotRegular,
            "{link}"
        );
    }

    assert_eq!(
        inspect(root.path(), &path("/etc/folder.conf")),
        Found::Folder {
            files: vec!["app.conf".to_owned()],
            link: Some(path("/etc/real"))
        }
    );
    assert_eq!(
        inspect(root.path(), &path("/etc/pipe.conf")),
        Found::Link {
            target: "/etc/real/pipe".to_owned(),
            reason: FetchRefusal::NotRegular
        }
    );
}

#[test]
fn a_link_into_a_forbidden_place_is_neither_read_nor_written() {
    let root = system();

    for (name, target) in [
        ("os-release", "usr/lib/os-release"),
        ("resolv.conf", "run/systemd/resolve/stub-resolv.conf"),
        ("fstab", "nix/store/abc-etc-fstab"),
        ("mounts", "proc/self/mounts"),
        ("notes", "sys/kernel/notes"),
        ("disk", "dev/disk"),
        ("history", "root/.bash_history"),
        ("hashes", "etc/shadow"),
    ] {
        let victim = root.path().join(target);
        let link = format!("/etc/{name}");

        fs::create_dir_all(victim.parent().unwrap()).unwrap();
        fs::write(&victim, OLD).unwrap();
        symlink(format!("/{target}"), root.path().join("etc").join(name)).unwrap();

        assert_eq!(
            refused(read_config_file(root.path(), &path(&link))),
            FetchRefusal::Outside,
            "{link}"
        );
        assert_eq!(write_refused(&root, &link), FetchRefusal::Outside, "{link}");
        assert_eq!(fs::read_to_string(&victim).unwrap(), OLD, "{link}");
        assert_eq!(
            inspect(root.path(), &path(&link)),
            Found::Link {
                target: format!("/{target}"),
                reason: FetchRefusal::Outside
            },
            "{link}"
        );
    }

    assert!(!root.path().join(BACKUPS).exists(), "a backup was written");
}

/// What an attacker with a moment between the look and the open could do:
/// put a link where the real file was.
#[test]
fn a_link_swapped_in_after_the_resolve_is_not_followed() {
    let root = system();
    let real = resolve(root.path(), &path("/etc/app.conf")).unwrap();
    let victim = root.path().join("usr/lib/victim");
    let target = root.path().join("etc/real/app.conf");

    assert_eq!(real.path, path("/etc/real/app.conf"));
    fs::write(&victim, OLD).unwrap();
    fs::remove_file(&target).unwrap();
    symlink(&victim, &target).unwrap();

    assert_eq!(
        refused(read_real(root.path(), &real.path)),
        FetchRefusal::Link
    );
    assert_eq!(write_real_refused(&root, &real.path), FetchRefusal::Link);
    assert_eq!(fs::read_to_string(&victim).unwrap(), OLD);
    assert!(is_link(&target), "the swapped link was replaced");
}

#[test]
fn a_folder_swapped_for_a_link_after_the_resolve_is_not_entered() {
    let root = system();
    let real = resolve(root.path(), &path("/etc/app.conf")).unwrap();
    let elsewhere = root.path().join("usr/lib/elsewhere");

    fs::create_dir_all(&elsewhere).unwrap();
    fs::write(elsewhere.join("app.conf"), OLD).unwrap();
    fs::rename(root.path().join("etc/real"), root.path().join("etc/moved")).unwrap();
    symlink(&elsewhere, root.path().join("etc/real")).unwrap();

    assert_eq!(
        refused(read_real(root.path(), &real.path)),
        FetchRefusal::Link
    );
    assert_eq!(write_real_refused(&root, &real.path), FetchRefusal::Link);
    assert_eq!(fs::read_to_string(elsewhere.join("app.conf")).unwrap(), OLD);
    assert_eq!(fs::read_dir(&elsewhere).unwrap().count(), 1);
}

#[test]
fn the_step_names_the_file_that_was_written() {
    let root = system();
    let mut events = Vec::new();
    let write = |file: &str| ConfigWrite {
        path: path(file),
        content: text(NEW),
        original: Fingerprint::of(OLD.as_bytes()),
    };

    assert_eq!(
        execute(
            root.path(),
            StepId::new(1),
            &write("/etc/app.conf"),
            &mut events
        ),
        Outcome::Succeeded
    );
    assert_eq!(
        printed(&events),
        "wrote /etc/real/app.conf (where the link /etc/app.conf leads), the previous version is \
         /var/lib/garuda-toolbox/backups/etc/real/app.conf\r\n"
    );

    symlink("/usr/lib/os-release", root.path().join("etc/os-release")).unwrap();
    fs::write(root.path().join("usr/lib/os-release"), OLD).unwrap();

    let mut outside = Vec::new();

    assert_eq!(
        execute(
            root.path(),
            StepId::new(2),
            &write("/etc/os-release"),
            &mut outside
        ),
        Outcome::Failed {
            cause: Failure::Unknown
        }
    );
    let text = printed(&outside);

    assert!(
        text.contains(
            "/etc/os-release cannot be replaced: it is a link to a file outside the folders"
        ),
        "{text}"
    );
}

#[test]
fn a_link_whose_way_the_user_may_not_walk_is_hidden_not_refused() {
    if rustix::process::geteuid().is_root() {
        return;
    }

    let root = system();
    let private = root.path().join("etc/real");

    fs::set_permissions(&private, fs::Permissions::from_mode(0o000)).unwrap();

    let found = inspect(root.path(), &path("/etc/app.conf"));
    let read = read_config_file(root.path(), &path("/etc/app.conf"));

    fs::set_permissions(&private, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(found, Found::Hidden);
    assert!(matches!(
        read,
        Err(ReadError::Failed(error)) if error.kind() == std::io::ErrorKind::PermissionDenied
    ));
}
