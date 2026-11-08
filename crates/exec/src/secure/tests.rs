use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;

use rustix::io::Errno;

use super::{
    Attributes, Unreadable, create_dirs, open_dir, read_file, regular_files, remove_file,
    replace_system_file,
};

fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();

    names.sort();
    names
}

#[test]
fn a_file_is_replaced_with_mode_0644() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc/conf")).unwrap();
    fs::write(root.path().join("etc/conf/list"), "old").unwrap();
    fs::set_permissions(
        root.path().join("etc/conf/list"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();

    let dir = open_dir(root.path(), Path::new("etc/conf")).unwrap();
    replace_system_file(&dir, "list", b"new", None).unwrap();

    let target = root.path().join("etc/conf/list");
    assert_eq!(fs::read_to_string(&target).unwrap(), "new");
    assert_eq!(
        fs::metadata(&target).unwrap().permissions().mode() & 0o777,
        0o644
    );
    assert_eq!(names(&root.path().join("etc/conf")), ["list"]);
}

#[test]
fn the_old_file_becomes_the_backup() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc")).unwrap();
    fs::write(root.path().join("etc/list"), "old").unwrap();
    fs::write(root.path().join("etc/list.bak"), "older").unwrap();

    let dir = open_dir(root.path(), Path::new("etc")).unwrap();
    let backed_up = replace_system_file(&dir, "list", b"new", Some("list.bak")).unwrap();

    assert!(backed_up);
    assert_eq!(
        fs::read_to_string(root.path().join("etc/list")).unwrap(),
        "new"
    );
    assert_eq!(
        fs::read_to_string(root.path().join("etc/list.bak")).unwrap(),
        "old"
    );
    assert_eq!(names(&root.path().join("etc")), ["list", "list.bak"]);
}

#[test]
fn a_first_install_has_no_backup() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc")).unwrap();

    let dir = open_dir(root.path(), Path::new("etc")).unwrap();
    let backed_up = replace_system_file(&dir, "list", b"new", Some("list.bak")).unwrap();

    assert!(!backed_up);
    assert_eq!(names(&root.path().join("etc")), ["list"]);
}

#[test]
fn a_symlinked_directory_on_the_way_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc")).unwrap();
    std::os::unix::fs::symlink(elsewhere.path(), root.path().join("etc/conf")).unwrap();

    open_dir(root.path(), Path::new("etc/conf")).unwrap_err();
    assert_eq!(names(elsewhere.path()), Vec::<String>::new());
}

#[test]
fn a_symlink_as_the_file_is_replaced_not_followed() {
    let root = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    let victim = elsewhere.path().join("shadow");
    fs::write(&victim, "secret").unwrap();
    fs::create_dir_all(root.path().join("etc")).unwrap();
    std::os::unix::fs::symlink(&victim, root.path().join("etc/list")).unwrap();

    let dir = open_dir(root.path(), Path::new("etc")).unwrap();
    replace_system_file(&dir, "list", b"new", Some("list.bak")).unwrap();

    assert_eq!(fs::read_to_string(&victim).unwrap(), "secret");
    let target = root.path().join("etc/list");
    assert!(!fs::symlink_metadata(&target).unwrap().is_symlink());
    assert_eq!(fs::read_to_string(target).unwrap(), "new");
}

/// A link that stays below the root is refused too: the folder it leads
/// to may be one another user can write.
#[test]
fn a_symlink_that_stays_inside_the_root_is_refused() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc/real/deeper")).unwrap();
    std::os::unix::fs::symlink("real", root.path().join("etc/link")).unwrap();

    open_dir(root.path(), Path::new("etc/link")).unwrap_err();
    open_dir(root.path(), Path::new("etc/link/deeper")).unwrap_err();
    create_dirs(root.path(), Path::new("etc/link/deeper/new")).unwrap_err();
    open_dir(root.path(), Path::new("etc/real/deeper")).unwrap();
}

#[test]
fn an_absolute_symlink_on_the_way_is_refused() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc")).unwrap();
    std::os::unix::fs::symlink("/", root.path().join("etc/up")).unwrap();

    open_dir(root.path(), Path::new("etc/up/tmp")).unwrap_err();
    open_dir(root.path(), Path::new("etc/up")).unwrap_err();
}

#[test]
fn only_plain_relative_directories_are_opened() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc")).unwrap();

    open_dir(root.path(), Path::new("etc/../etc")).unwrap_err();
    open_dir(root.path(), Path::new("/etc")).unwrap_err();
    open_dir(root.path(), Path::new("missing")).unwrap_err();
}

#[test]
fn missing_folders_are_created_with_mode_0755() {
    let root = tempfile::tempdir().unwrap();

    let dir = create_dirs(root.path(), Path::new("etc/garuda/garuda-update")).unwrap();
    replace_system_file(&dir, "config", b"A=1\n", None).unwrap();

    let folder = root.path().join("etc/garuda/garuda-update");
    assert_eq!(
        fs::metadata(&folder).unwrap().permissions().mode() & 0o777,
        0o755
    );
    assert_eq!(fs::read_to_string(folder.join("config")).unwrap(), "A=1\n");
    create_dirs(root.path(), Path::new("etc/garuda/garuda-update")).unwrap();
}

#[test]
fn creating_folders_does_not_follow_a_symlink() {
    let root = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc")).unwrap();
    std::os::unix::fs::symlink(elsewhere.path(), root.path().join("etc/garuda")).unwrap();

    create_dirs(root.path(), Path::new("etc/garuda/garuda-update")).unwrap_err();
    assert_eq!(names(elsewhere.path()), Vec::<String>::new());
}

#[test]
fn a_missing_file_reads_as_none() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("present"), "text").unwrap();
    let dir = open_dir(root.path(), Path::new("")).unwrap();

    assert_eq!(
        read_file(&dir, "present").unwrap().as_deref(),
        Some(&b"text"[..])
    );
    assert_eq!(read_file(&dir, "missing").unwrap(), None);
}

#[test]
fn reading_refuses_a_symlink() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("secret"), "text").unwrap();
    std::os::unix::fs::symlink(root.path().join("secret"), root.path().join("link")).unwrap();
    let dir = open_dir(root.path(), Path::new("")).unwrap();

    read_file(&dir, "link").unwrap_err();
}

#[test]
fn removing_reports_whether_there_was_a_file() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("drop-in.conf"), "x").unwrap();
    let dir = open_dir(root.path(), Path::new("")).unwrap();

    assert!(remove_file(&dir, "drop-in.conf").unwrap());
    assert!(!remove_file(&dir, "drop-in.conf").unwrap());
    assert_eq!(names(root.path()), Vec::<String>::new());
}

#[test]
fn errnos_become_the_reason_the_content_is_missing() {
    assert!(matches!(
        Unreadable::from(Errno::NOENT),
        Unreadable::Missing
    ));
    assert!(matches!(
        Unreadable::from(Errno::NXIO),
        Unreadable::NotRegular
    ));
    assert!(matches!(Unreadable::from(Errno::LOOP), Unreadable::Link));
    assert!(matches!(Unreadable::from(Errno::NOTDIR), Unreadable::Link));
    assert!(matches!(
        Unreadable::from(Errno::PIPE),
        Unreadable::Failed(_)
    ));
}

#[test]
fn only_plain_relative_folders_are_made() {
    let root = tempfile::tempdir().unwrap();

    create_dirs(root.path(), Path::new("/abs")).unwrap_err();
    create_dirs(root.path(), Path::new("etc/../etc")).unwrap_err();

    fs::write(root.path().join("file"), "").unwrap();
    create_dirs(root.path(), Path::new("file/below")).unwrap_err();
}

#[test]
fn removing_through_a_file_is_an_error() {
    let root = tempfile::tempdir().unwrap();
    let dir = open_dir(root.path(), Path::new("")).unwrap();

    fs::write(root.path().join("file"), "").unwrap();

    remove_file(&dir, "file/below").unwrap_err();
}

#[test]
fn a_failed_replace_leaves_no_temporary_file() {
    let root = tempfile::tempdir().unwrap();
    let dir = open_dir(root.path(), Path::new("")).unwrap();

    fs::create_dir_all(root.path().join("kept/filled")).unwrap();
    fs::write(root.path().join("kept/filled/file"), "").unwrap();
    fs::write(root.path().join("plain"), "old").unwrap();
    fs::create_dir_all(root.path().join("taken/filled")).unwrap();
    fs::write(root.path().join("taken/filled/file"), "").unwrap();

    replace_system_file(&dir, "kept", b"new", None).unwrap_err();
    replace_system_file(&dir, "plain", b"new", Some("taken")).unwrap_err();

    assert!(
        !names(root.path()).iter().any(|name| name.contains(".tmp-")),
        "no temporary file is left"
    );

    let attributes = Attributes {
        mode: 0o600,
        owner: rustix::process::geteuid().as_raw(),
        group: rustix::process::getegid().as_raw(),
    };

    toolbox_fs::replace_at(&dir, "kept", b"new", &attributes.replace()).unwrap_err();
    assert!(
        !names(root.path()).iter().any(|name| name.contains(".tmp-")),
        "no temporary file is left"
    );
}

#[test]
fn names_that_are_no_text_are_left_out() {
    let root = tempfile::tempdir().unwrap();
    let dir = open_dir(root.path(), Path::new("")).unwrap();

    fs::write(root.path().join(OsStr::from_bytes(b"\xff\xfe")), "").unwrap();
    fs::write(root.path().join("plain"), "").unwrap();

    assert_eq!(regular_files(&dir).unwrap(), ["plain"]);
}
