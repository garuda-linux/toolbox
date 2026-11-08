mod links;

use std::error::Error as _;
use std::fs;
use std::os::unix::fs::{PermissionsExt as _, symlink};
use std::path::Path;

use tempfile::TempDir;
use toolbox_plan::{ConfigFilePath, ConfigText, FetchRefusal, MAX_CONFIG_BYTES};

use super::{ConfigRead, Found, ReadError, inspect, look, read_config_file};
use crate::secure::Unreadable;

pub(super) const BACKUPS: &str = "var/lib/garuda-toolbox/backups";

/// A root folder with `etc/fstab` and `etc/conf.d`.
fn system() -> TempDir {
    let root = tempfile::tempdir().unwrap();

    fs::create_dir_all(root.path().join("etc/conf.d")).unwrap();
    fs::write(root.path().join("etc/fstab"), "old = 1\n").unwrap();
    root
}

pub(super) fn path(text: &str) -> ConfigFilePath {
    ConfigFilePath::new(text).unwrap()
}

pub(super) fn text(content: &str) -> ConfigText {
    ConfigText::new(content).unwrap()
}

fn refused<T>(result: Result<T, ReadError>) -> FetchRefusal
where
    T: std::fmt::Debug,
{
    match result.unwrap_err() {
        ReadError::Refused(reason) => reason,
        ReadError::Failed(error) => panic!("failed instead of refused: {error}"),
    }
}

fn fifo(path: &Path) {
    rustix::fs::mknodat(
        rustix::fs::CWD,
        path,
        rustix::fs::FileType::Fifo,
        rustix::fs::Mode::from_raw_mode(0o644),
        0,
    )
    .unwrap();
}

#[test]
fn a_regular_text_file_is_read() {
    let root = system();

    assert_eq!(
        read_config_file(root.path(), &path("/etc/fstab")).unwrap(),
        ConfigRead {
            path: path("/etc/fstab"),
            content: text("old = 1\n")
        }
    );
}

#[test]
fn a_missing_file_or_folder_is_an_answer() {
    let root = system();

    assert_eq!(
        refused(read_config_file(root.path(), &path("/etc/nothing"))),
        FetchRefusal::Missing
    );
    assert_eq!(
        refused(read_config_file(root.path(), &path("/etc/none/file"))),
        FetchRefusal::Missing
    );
}

#[test]
fn a_file_in_a_linked_folder_is_read_where_it_really_is() {
    let root = system();

    fs::write(root.path().join("etc/conf.d/file"), "inside\n").unwrap();
    symlink("conf.d", root.path().join("etc/linked")).unwrap();

    assert_eq!(
        read_config_file(root.path(), &path("/etc/linked/file")).unwrap(),
        ConfigRead {
            path: path("/etc/conf.d/file"),
            content: text("inside\n")
        }
    );
    assert_eq!(
        inspect(root.path(), &path("/etc/linked/file")),
        Found::File {
            link: Some(path("/etc/conf.d/file"))
        }
    );
    assert_eq!(
        refused(read_config_file(root.path(), &path("/etc/linked/none"))),
        FetchRefusal::Missing
    );
}

#[test]
fn folders_fifos_and_sockets_are_not_read() {
    let root = system();

    fifo(&root.path().join("etc/pipe"));

    let _socket = std::os::unix::net::UnixListener::bind(root.path().join("etc/socket")).unwrap();

    for name in ["/etc/conf.d", "/etc/pipe", "/etc/socket"] {
        assert_eq!(
            refused(read_config_file(root.path(), &path(name))),
            FetchRefusal::NotRegular,
            "{name}"
        );
    }
}

#[test]
fn an_oversized_file_is_not_read() {
    let root = system();

    fs::write(
        root.path().join("etc/big"),
        vec![b'x'; MAX_CONFIG_BYTES + 1],
    )
    .unwrap();
    fs::write(root.path().join("etc/full"), vec![b'x'; MAX_CONFIG_BYTES]).unwrap();
    symlink("big", root.path().join("etc/big-link")).unwrap();

    for name in ["/etc/big", "/etc/big-link"] {
        assert_eq!(
            refused(read_config_file(root.path(), &path(name))),
            FetchRefusal::TooLarge,
            "{name}"
        );
    }

    read_config_file(root.path(), &path("/etc/full")).unwrap();
}

#[test]
fn binary_files_are_not_read() {
    let root = system();

    fs::write(root.path().join("etc/latin1"), [b'g', b'r', 0xfc, b'n']).unwrap();
    fs::write(root.path().join("etc/nul"), b"a\0b").unwrap();
    symlink("nul", root.path().join("etc/nul-link")).unwrap();

    for name in ["/etc/latin1", "/etc/nul", "/etc/nul-link"] {
        assert_eq!(
            refused(read_config_file(root.path(), &path(name))),
            FetchRefusal::NotText,
            "{name}"
        );
    }
}

#[test]
fn inspect_tells_files_folders_links_and_the_rest_apart() {
    let root = system();

    fs::write(root.path().join("etc/conf.d/b.conf"), "").unwrap();
    fs::write(root.path().join("etc/conf.d/a.conf"), "").unwrap();
    fs::create_dir_all(root.path().join("etc/conf.d/sub")).unwrap();
    symlink("a.conf", root.path().join("etc/conf.d/link.conf")).unwrap();
    symlink("fstab", root.path().join("etc/mtab")).unwrap();
    fifo(&root.path().join("etc/pipe"));

    assert_eq!(
        inspect(root.path(), &path("/etc/fstab")),
        Found::File { link: None }
    );
    assert_eq!(
        inspect(root.path(), &path("/etc/conf.d")),
        Found::Folder {
            files: vec!["a.conf".to_owned(), "b.conf".to_owned()],
            link: None
        }
    );
    assert_eq!(
        inspect(root.path(), &path("/etc/mtab")),
        Found::File {
            link: Some(path("/etc/fstab"))
        }
    );
    assert_eq!(inspect(root.path(), &path("/etc/nothing")), Found::Missing);
    assert_eq!(
        inspect(root.path(), &path("/etc/none/file")),
        Found::Missing
    );
    assert_eq!(inspect(root.path(), &path("/etc/pipe")), Found::Other);
    assert_eq!(
        inspect(root.path(), &path("/boot/grub/grub.cfg")),
        Found::Missing
    );
}

#[test]
fn a_read_error_says_why_in_the_words_of_the_console() {
    let failed = ReadError::Failed(std::io::Error::other("denied"));

    assert_eq!(
        ReadError::Refused(FetchRefusal::TooLarge).to_string(),
        "the file is too large"
    );
    assert!(
        ReadError::Refused(FetchRefusal::TooLarge)
            .source()
            .is_none()
    );
    assert_eq!(failed.to_string(), "denied");
    assert_eq!(failed.source().unwrap().to_string(), "denied");
}

#[test]
fn missing_content_maps_to_a_missing_file() {
    assert!(matches!(
        ReadError::from(Unreadable::Missing),
        ReadError::Refused(FetchRefusal::Missing)
    ));
}

#[test]
fn a_linked_folder_reads_as_missing() {
    let root = system();

    fs::create_dir_all(root.path().join("elsewhere")).unwrap();
    fs::remove_dir_all(root.path().join("etc")).unwrap();
    symlink(root.path().join("elsewhere"), root.path().join("etc")).unwrap();
    fs::write(root.path().join("elsewhere/fstab"), "x\n").unwrap();

    assert_eq!(look(root.path(), &path("/etc/fstab")), Found::Missing);
    assert_eq!(
        inspect(root.path(), &path("/etc/fstab")),
        Found::Link {
            target: "/etc/fstab".to_owned(),
            reason: FetchRefusal::Outside,
        }
    );
}

#[test]
fn unreadable_folders_hide_their_files() {
    if rustix::process::geteuid().is_root() {
        return;
    }

    let root = system();

    fs::set_permissions(root.path().join("etc"), fs::Permissions::from_mode(0o000)).unwrap();

    assert_eq!(look(root.path(), &path("/etc/fstab")), Found::Hidden);

    fs::set_permissions(root.path().join("etc"), fs::Permissions::from_mode(0o755)).unwrap();
}
