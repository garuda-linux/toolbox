use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;

use toolbox_config::Paths;

use super::{log_dir, private_folder, temporary};

#[test]
fn the_state_folder_keeps_the_log_when_there_is_a_home() {
    let paths = Paths::from_env(|key| (key == "HOME").then(|| OsString::from("/home/nico")));

    assert_eq!(
        log_dir(&paths),
        Some(PathBuf::from("/home/nico/.local/state/garuda-toolbox"))
    );
}

#[test]
fn without_a_home_a_private_temporary_folder_keeps_the_log() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join("garuda-toolbox");

    assert_eq!(temporary(&dir), Some(dir.clone()));
    assert!(dir.is_dir());
}

#[test]
fn an_unsafe_temporary_folder_leaves_stderr_alone() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join("garuda-toolbox");

    fs::write(&dir, "").unwrap();

    assert_eq!(temporary(&dir), None);
}

#[test]
fn a_fresh_private_folder_is_made_for_the_owner_alone() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join("garuda-toolbox");

    private_folder(&dir).unwrap();

    let meta = fs::symlink_metadata(&dir).unwrap();
    assert!(meta.is_dir());
    assert_eq!(meta.permissions().mode() & 0o777, 0o700);
}

#[test]
fn an_existing_private_folder_is_kept() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join("garuda-toolbox");
    fs::create_dir_all(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();

    private_folder(&dir).unwrap();
}

#[test]
fn a_folder_others_can_write_is_refused() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join("garuda-toolbox");
    fs::create_dir_all(&dir).unwrap();

    for mode in [0o777, 0o770, 0o702] {
        fs::set_permissions(&dir, fs::Permissions::from_mode(mode)).unwrap();

        assert!(private_folder(&dir).is_err(), "{mode:o}");
    }
}

/// Another user could point the link anywhere, so even a link to a good
/// folder is refused.
#[test]
fn a_symlink_is_refused() {
    let temp = tempfile::tempdir().unwrap();
    let real = temp.path().join("real");
    fs::create_dir_all(&real).unwrap();
    fs::set_permissions(&real, fs::Permissions::from_mode(0o700)).unwrap();
    let link = temp.path().join("garuda-toolbox");
    std::os::unix::fs::symlink(&real, &link).unwrap();

    assert!(private_folder(&link).is_err());
}

#[test]
fn a_file_in_the_way_is_refused() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join("garuda-toolbox");
    fs::write(&dir, "").unwrap();

    assert!(private_folder(&dir).is_err());
}
