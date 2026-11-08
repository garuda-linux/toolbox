use std::fs;
use std::os::fd::OwnedFd;
use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
use std::path::Path;

use rustix::fs::{Mode, OFlags};

use super::{Owner, Permissions, Replace, replace, replace_at};

fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();

    names.sort();
    names
}

fn open(dir: &Path) -> OwnedFd {
    rustix::fs::open(dir, OFlags::DIRECTORY | OFlags::RDONLY, Mode::empty()).unwrap()
}

fn mode(path: &Path) -> u32 {
    fs::metadata(path).unwrap().permissions().mode() & 0o7777
}

fn myself(dir: &Path) -> Owner {
    let meta = fs::metadata(dir).unwrap();

    Owner {
        user: meta.uid(),
        group: meta.gid(),
    }
}

#[test]
fn a_file_is_replaced_with_exactly_the_given_mode() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("list");

    fs::write(&target, "old").unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();

    let replaced = replace_at(
        open(root.path()),
        "list",
        b"new",
        &Replace::new(Permissions::Exactly(0o644)),
    )
    .unwrap();

    assert!(!replaced.synced().unwrap());
    assert_eq!(fs::read_to_string(&target).unwrap(), "new");
    assert_eq!(mode(&target), 0o644);
    assert_eq!(names(root.path()), ["list"]);
}

#[test]
fn the_old_file_becomes_the_backup() {
    let root = tempfile::tempdir().unwrap();

    fs::write(root.path().join("list"), "old").unwrap();
    fs::write(root.path().join("list.bak"), "older").unwrap();

    let how = Replace::new(Permissions::Exactly(0o644)).with_backup("list.bak");
    let kept = replace_at(open(root.path()), "list", b"new", &how)
        .unwrap()
        .synced()
        .unwrap();

    assert!(kept);
    assert_eq!(fs::read_to_string(root.path().join("list")).unwrap(), "new");
    assert_eq!(
        fs::read_to_string(root.path().join("list.bak")).unwrap(),
        "old"
    );
    assert_eq!(names(root.path()), ["list", "list.bak"]);
}

#[test]
fn a_first_install_has_no_backup() {
    let root = tempfile::tempdir().unwrap();
    let how = Replace::new(Permissions::Exactly(0o644)).with_backup("list.bak");
    let kept = replace_at(open(root.path()), "list", b"new", &how)
        .unwrap()
        .synced()
        .unwrap();

    assert!(!kept);
    assert_eq!(names(root.path()), ["list"]);
}

#[test]
fn a_symlink_as_the_file_is_replaced_not_followed() {
    let root = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    let victim = elsewhere.path().join("shadow");

    fs::write(&victim, "secret").unwrap();
    std::os::unix::fs::symlink(&victim, root.path().join("list")).unwrap();

    let how = Replace::new(Permissions::Exactly(0o644)).with_backup("list.bak");

    replace_at(open(root.path()), "list", b"new", &how).unwrap();

    let target = root.path().join("list");

    assert_eq!(fs::read_to_string(&victim).unwrap(), "secret");
    assert!(!fs::symlink_metadata(&target).unwrap().is_symlink());
    assert_eq!(fs::read_to_string(target).unwrap(), "new");
}

#[test]
fn an_owner_and_mode_are_set_on_the_new_file() {
    let root = tempfile::tempdir().unwrap();
    let owner = myself(root.path());
    let how = Replace::new(Permissions::Exactly(0o640)).owned_by(owner);

    replace_at(open(root.path()), "sudoers", b"x", &how)
        .unwrap()
        .synced()
        .unwrap();

    let target = root.path().join("sudoers");
    let meta = fs::metadata(&target).unwrap();

    assert_eq!(mode(&target), 0o640);
    assert_eq!(meta.uid(), owner.user);
    assert_eq!(meta.gid(), owner.group);
}

#[test]
fn a_kept_mode_wins_over_the_given_one() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("settings.toml");

    fs::write(&target, "old").unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();

    let how = Replace::new(Permissions::Exactly(0o644)).keeping_mode();

    replace(&target, b"new", &how).unwrap().synced().unwrap();

    assert_eq!(mode(&target), 0o600);
    assert_eq!(fs::read_to_string(&target).unwrap(), "new");
}

#[test]
fn without_a_file_to_keep_the_given_mode_applies() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("settings.toml");
    let how = Replace::new(Permissions::Exactly(0o644)).keeping_mode();

    replace(&target, b"new", &how).unwrap().synced().unwrap();

    assert_eq!(mode(&target), 0o644);
}

#[test]
fn an_umask_mode_is_what_the_umask_leaves() {
    let root = tempfile::tempdir().unwrap();
    let probe = root.path().join("probe");

    // What the umask of this process leaves of 0666, seen on a plain file.
    fs::write(&probe, "").unwrap();

    let target = root.path().join("kwinrc");

    replace(&target, b"x", &Replace::new(Permissions::Umask(0o666)))
        .unwrap()
        .synced()
        .unwrap();

    assert_eq!(mode(&target), mode(&probe));
}

#[test]
fn the_path_form_creates_missing_folders() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("a/b/settings.toml");

    replace(&target, b"x", &Replace::new(Permissions::Exactly(0o644)))
        .unwrap()
        .synced()
        .unwrap();

    assert_eq!(fs::read_to_string(&target).unwrap(), "x");
    assert_eq!(names(&root.path().join("a/b")), ["settings.toml"]);
}

#[test]
fn a_path_without_a_file_name_is_refused() {
    let root = tempfile::tempdir().unwrap();

    replace(
        root.path(),
        b"x",
        &Replace::new(Permissions::Exactly(0o644)),
    )
    .unwrap_err();
    replace(
        &root.path().join(".."),
        b"x",
        &Replace::new(Permissions::Exactly(0o644)),
    )
    .unwrap_err();
}

/// The rename needs only write and search rights on the folder. Its sync
/// needs it open for reading, so only that fails.
#[test]
fn a_folder_that_cannot_be_read_takes_the_file_but_not_the_sync() {
    let root = tempfile::tempdir().unwrap();
    let folder = root.path().join("locked");

    fs::create_dir_all(&folder).unwrap();
    fs::set_permissions(&folder, fs::Permissions::from_mode(0o300)).unwrap();

    // Root opens any folder, so nothing fails there.
    let opens = fs::File::open(&folder).is_ok();
    let replaced = replace(
        &folder.join("settings.toml"),
        b"new",
        &Replace::new(Permissions::Exactly(0o644)),
    );

    fs::set_permissions(&folder, fs::Permissions::from_mode(0o700)).unwrap();

    if !opens {
        replaced.unwrap().synced().unwrap_err();
        assert_eq!(
            fs::read_to_string(folder.join("settings.toml")).unwrap(),
            "new"
        );
    }
}

#[test]
fn a_failed_replace_leaves_no_temporary_file() {
    let root = tempfile::tempdir().unwrap();
    let how = Replace::new(Permissions::Exactly(0o644));

    fs::create_dir_all(root.path().join("kept/filled")).unwrap();
    fs::write(root.path().join("kept/filled/file"), "").unwrap();
    fs::write(root.path().join("plain"), "old").unwrap();
    fs::create_dir_all(root.path().join("taken/filled")).unwrap();
    fs::write(root.path().join("taken/filled/file"), "").unwrap();

    replace_at(open(root.path()), "kept", b"new", &how).unwrap_err();
    replace_at(
        open(root.path()),
        "plain",
        b"new",
        &how.clone().with_backup("taken"),
    )
    .unwrap_err();
    replace_at(
        open(root.path()),
        "kept",
        b"new",
        &how.owned_by(myself(root.path())),
    )
    .unwrap_err();

    assert!(
        !names(root.path()).iter().any(|name| name.contains(".tmp-")),
        "no temporary file is left"
    );
    assert_eq!(
        fs::read_to_string(root.path().join("plain")).unwrap(),
        "old"
    );
}

#[test]
fn two_temporary_names_for_one_file_differ() {
    assert_ne!(super::temp_name("list"), super::temp_name("list"));
    assert!(super::temp_name("list").starts_with(".list.tmp-"));
}
