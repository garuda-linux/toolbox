use std::fs;
use std::os::unix::fs::{PermissionsExt as _, symlink};

use super::{write_atomic, write_private};

#[test]
fn writes_into_missing_folders_and_replaces_old_content() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a/b/changes.json");

    write_atomic(&path, b"old").unwrap();
    write_atomic(&path, b"new").unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "new");
    assert_eq!(
        fs::read_dir(path.parent().unwrap()).unwrap().count(),
        1,
        "no temporary file left"
    );
}

#[test]
fn keeps_the_destination_mode_and_uses_0600_for_new_files() {
    let dir = tempfile::tempdir().unwrap();
    let kept = dir.path().join("kept.toml");
    fs::write(&kept, b"old").unwrap();
    fs::set_permissions(&kept, fs::Permissions::from_mode(0o640)).unwrap();

    write_atomic(&kept, b"new").unwrap();

    assert_eq!(fs::read_to_string(&kept).unwrap(), "new");
    assert_eq!(
        fs::metadata(&kept).unwrap().permissions().mode() & 0o777,
        0o640
    );

    let fresh = dir.path().join("fresh.toml");
    write_atomic(&fresh, b"new").unwrap();

    assert_eq!(
        fs::metadata(&fresh).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn a_symlinked_file_stays_a_link_and_its_target_gets_the_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let real = dir.path().join("dotfiles/settings.toml");
    fs::create_dir_all(real.parent().unwrap()).unwrap();
    fs::write(&real, b"old").unwrap();
    let link = dir.path().join("config/settings.toml");
    fs::create_dir_all(link.parent().unwrap()).unwrap();
    symlink("../dotfiles/settings.toml", &link).unwrap();

    write_atomic(&link, b"new").unwrap();

    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(fs::read_to_string(&real).unwrap(), "new");
}

#[test]
fn a_dangling_link_creates_its_target() {
    let dir = tempfile::tempdir().unwrap();
    let real = dir.path().join("elsewhere/settings.toml");
    let link = dir.path().join("settings.toml");
    symlink(&real, &link).unwrap();

    write_atomic(&link, b"new").unwrap();

    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(fs::read_to_string(&real).unwrap(), "new");
}

#[test]
fn a_link_loop_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let first = dir.path().join("a.toml");
    let second = dir.path().join("b.toml");
    symlink(&second, &first).unwrap();
    symlink(&first, &second).unwrap();

    assert!(write_atomic(&first, b"new").is_err());
}

/// A folder without read permission takes the rename but cannot be opened
/// for the final sync. The file is in place by then, so the save counts.
#[test]
fn a_folder_that_cannot_be_synced_still_saves() {
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path().join("locked");
    fs::create_dir_all(&folder).unwrap();
    let path = folder.join("settings.toml");
    fs::set_permissions(&folder, fs::Permissions::from_mode(0o300)).unwrap();

    // Root opens any folder, so the sync would not fail there.
    let opens = fs::File::open(&folder).is_ok();
    let result = write_atomic(&path, b"new");

    fs::set_permissions(&folder, fs::Permissions::from_mode(0o700)).unwrap();

    if !opens {
        result.unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "new");
    }
}

/// The queue names users and packages, so an older file of 0644 is
/// tightened on the next save.
#[test]
fn a_private_file_is_0600_whatever_it_was() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("changes.json");
    fs::write(&path, b"old").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

    write_private(&path, b"new").unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "new");
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
}
