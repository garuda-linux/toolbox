use std::fs;
use std::os::unix::fs::{PermissionsExt as _, symlink};
use std::path::Path;

use toolbox_plan::{ConfigText, Fingerprint, HomePath, HomeWrite};

use super::write_home_file;
use crate::test_support::mode;

const FILE: &str = ".config/kwinrc";
const BACKUP: &str = ".local/state/garuda-toolbox/backups/.config/kwinrc";

/// The `kwinrc` of a used Plasma session on a Garuda install.
const OLD: &str = toolbox_plan::testing::user_config::INSTALLED_KWINRC;

/// Edit: the drag of the wobbly windows effect lowered from 85 to 50, the
/// one setting a user changes in the editor.
fn new() -> String {
    let new = OLD.replace("\nDrag=85\n", "\nDrag=50\n");

    assert_ne!(new, OLD);
    new
}

fn write(content: &str, original: &[u8]) -> HomeWrite {
    HomeWrite {
        path: HomePath::new(FILE).unwrap(),
        content: ConfigText::new(content).unwrap(),
        original: Fingerprint::of(original),
    }
}

fn home_with(content: &str) -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    fs::create_dir_all(home.path().join(".config")).unwrap();
    let file = home.path().join(FILE);
    fs::write(&file, content).unwrap();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o640)).unwrap();
    home
}

fn read(home: &Path, path: &str) -> String {
    fs::read_to_string(home.join(path)).unwrap()
}

#[test]
fn a_save_replaces_the_file_keeps_its_mode_and_the_old_version() {
    let home = home_with(OLD);

    write_home_file(home.path(), &write(&new(), OLD.as_bytes())).unwrap();

    assert_eq!(read(home.path(), FILE), new());
    assert_eq!(mode(&home.path().join(FILE)), 0o640);
    assert_eq!(read(home.path(), BACKUP), OLD);
    assert_eq!(mode(&home.path().join(BACKUP)), 0o600);
}

/// Edit: KWin itself switched the borderless setting off after the file
/// was read.
#[test]
fn a_file_that_changed_since_it_was_read_is_left_alone() {
    let changed = OLD.replace(
        "BorderlessMaximizedWindows=true",
        "BorderlessMaximizedWindows=false",
    );

    let home = home_with(&changed);

    let error = write_home_file(home.path(), &write(&new(), OLD.as_bytes())).unwrap_err();

    assert!(error.to_string().contains("changed"), "{error}");
    assert_eq!(read(home.path(), FILE), changed);
    assert!(!home.path().join(BACKUP).exists());
}

#[test]
fn a_missing_file_is_never_created() {
    let home = tempfile::tempdir().unwrap();

    assert!(write_home_file(home.path(), &write(&new(), b"")).is_err());
    assert!(!home.path().join(FILE).exists());
}

#[test]
fn a_linked_file_is_written_where_the_link_leads_and_the_link_stays() {
    let home = tempfile::tempdir().unwrap();
    let dotfiles = home.path().join("dotfiles");
    fs::create_dir_all(&dotfiles).unwrap();
    fs::create_dir_all(home.path().join(".config")).unwrap();
    fs::write(dotfiles.join("kwinrc"), OLD).unwrap();
    symlink(dotfiles.join("kwinrc"), home.path().join(FILE)).unwrap();

    write_home_file(home.path(), &write(&new(), OLD.as_bytes())).unwrap();

    let link = home.path().join(FILE);
    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(read(&dotfiles, "kwinrc"), new());
    assert_eq!(read(home.path(), BACKUP), OLD);
}
