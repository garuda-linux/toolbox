use std::fs;
use std::os::unix::fs::{PermissionsExt as _, symlink};
use std::path::Path;

use toolbox_plan::{ConfigText, Fingerprint, HomePath, HomeWrite};

use super::write_home_file;
use crate::testing::mode;

const BACKUP: &str = ".local/state/garuda-toolbox/backups/.config/app.conf";

fn write(content: &str, original: &[u8]) -> HomeWrite {
    HomeWrite {
        path: HomePath::new(".config/app.conf").unwrap(),
        content: ConfigText::new(content).unwrap(),
        original: Fingerprint::of(original),
    }
}

fn home_with(content: &str) -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    fs::create_dir_all(home.path().join(".config")).unwrap();
    let file = home.path().join(".config/app.conf");
    fs::write(&file, content).unwrap();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o640)).unwrap();
    home
}

fn read(home: &Path, path: &str) -> String {
    fs::read_to_string(home.join(path)).unwrap()
}

#[test]
fn a_save_replaces_the_file_keeps_its_mode_and_the_old_version() {
    let home = home_with("old\n");

    write_home_file(home.path(), &write("new\n", b"old\n")).unwrap();

    assert_eq!(read(home.path(), ".config/app.conf"), "new\n");
    assert_eq!(mode(&home.path().join(".config/app.conf")), 0o640);
    assert_eq!(read(home.path(), BACKUP), "old\n");
    assert_eq!(mode(&home.path().join(BACKUP)), 0o600);
}

#[test]
fn a_file_that_changed_since_it_was_read_is_left_alone() {
    let home = home_with("changed by someone else\n");

    let error = write_home_file(home.path(), &write("new\n", b"old\n")).unwrap_err();

    assert!(error.to_string().contains("changed"), "{error}");
    assert_eq!(
        read(home.path(), ".config/app.conf"),
        "changed by someone else\n"
    );
    assert!(!home.path().join(BACKUP).exists());
}

#[test]
fn a_missing_file_is_never_created() {
    let home = tempfile::tempdir().unwrap();

    assert!(write_home_file(home.path(), &write("new\n", b"")).is_err());
    assert!(!home.path().join(".config/app.conf").exists());
}

#[test]
fn a_linked_file_is_written_where_the_link_leads_and_the_link_stays() {
    let home = tempfile::tempdir().unwrap();
    let dotfiles = home.path().join("dotfiles");
    fs::create_dir_all(&dotfiles).unwrap();
    fs::create_dir_all(home.path().join(".config")).unwrap();
    fs::write(dotfiles.join("app.conf"), "old\n").unwrap();
    symlink(
        dotfiles.join("app.conf"),
        home.path().join(".config/app.conf"),
    )
    .unwrap();

    write_home_file(home.path(), &write("new\n", b"old\n")).unwrap();

    let link = home.path().join(".config/app.conf");
    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(read(&dotfiles, "app.conf"), "new\n");
    assert_eq!(read(home.path(), BACKUP), "old\n");
}
