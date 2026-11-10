use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use toolbox_plan::{
    DefaultEntry, GrubPicture, GrubSetting, GrubSettings, GrubTimeout, KernelParameters,
};

use super::edit_grub_defaults;
use crate::builtin::FileChange;
use crate::test_support::mode;

/// `/etc/default/grub` of a Garuda Mokka install.
const BEFORE: &str = toolbox_plan::testing::grub::INSTALLED_DEFAULT_GRUB;

/// What `settings` makes of it, line by line.
fn after() -> String {
    let mut after = BEFORE.to_owned();

    for (old, new) in [
        ("\nGRUB_DEFAULT=0\n", "\nGRUB_DEFAULT=saved\n"),
        ("\nGRUB_TIMEOUT=5\n", "\nGRUB_TIMEOUT=10\n"),
        (
            "\nGRUB_CMDLINE_LINUX_DEFAULT='quiet loglevel=3'\n",
            "\nGRUB_CMDLINE_LINUX_DEFAULT=\"quiet loglevel=3 splash\"\n",
        ),
        (
            "\n#GRUB_BACKGROUND=\"/path/to/wallpaper\"\n",
            "\nGRUB_BACKGROUND=\"/usr/share/my-wall.png\"\n",
        ),
        ("\n#GRUB_SAVEDEFAULT=true\n", "\nGRUB_SAVEDEFAULT=true\n"),
    ] {
        assert!(after.contains(old), "{old}");
        after = after.replacen(old, new, 1);
    }

    after
}

fn file(root: &Path) -> PathBuf {
    root.join("etc/default/grub")
}

fn system(content: &[u8]) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();

    fs::create_dir_all(root.path().join("etc/default")).unwrap();
    fs::write(file(root.path()), content).unwrap();
    root
}

fn settings() -> GrubSettings {
    GrubSettings::new(vec![
        GrubSetting::Default(DefaultEntry::Saved),
        GrubSetting::Timeout(GrubTimeout::new(10).unwrap()),
        GrubSetting::KernelParameters(KernelParameters::parse("quiet loglevel=3 splash").unwrap()),
        GrubSetting::SaveDefault(true),
        GrubSetting::Background(Some(GrubPicture::new("/usr/share/my-wall.png").unwrap())),
    ])
    .unwrap()
}

#[test]
fn several_settings_are_one_write_that_keeps_every_other_byte() {
    let root = system(BEFORE.as_bytes());

    assert_eq!(
        edit_grub_defaults(root.path(), &settings()).unwrap(),
        FileChange::Written
    );
    assert_eq!(fs::read_to_string(file(root.path())).unwrap(), after());
    assert_eq!(mode(&file(root.path())), 0o644);
}

#[test]
fn the_file_as_it_was_stays_beside_it_and_no_temporary_file_is_left() {
    let root = system(BEFORE.as_bytes());

    edit_grub_defaults(root.path(), &settings()).unwrap();

    let mut names: Vec<String> = fs::read_dir(root.path().join("etc/default"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();

    names.sort();
    assert_eq!(names, ["grub", "grub.bak"]);
    assert_eq!(
        fs::read_to_string(root.path().join("etc/default/grub.bak")).unwrap(),
        BEFORE
    );
}

#[test]
fn a_file_that_already_says_it_is_not_written() {
    let root = system(after().as_bytes());

    assert_eq!(
        edit_grub_defaults(root.path(), &settings()).unwrap(),
        FileChange::Unchanged
    );
    assert!(!root.path().join("etc/default/grub.bak").exists());
}

/// Without the file GRUB is not the boot loader. Creating it would make
/// the page of a systemd-boot system look editable.
#[test]
fn a_missing_file_is_never_created() {
    let root = tempfile::tempdir().unwrap();

    edit_grub_defaults(root.path(), &settings()).unwrap_err();
    assert!(!root.path().join("etc").exists());

    fs::create_dir_all(root.path().join("etc/default")).unwrap();

    let error = edit_grub_defaults(root.path(), &settings()).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    assert!(!file(root.path()).exists());
}

#[test]
fn a_symlinked_folder_or_file_is_refused() {
    let elsewhere = tempfile::tempdir().unwrap();
    let target = elsewhere.path().join("grub");
    fs::write(&target, BEFORE).unwrap();

    let linked_folder = tempfile::tempdir().unwrap();
    fs::create_dir_all(linked_folder.path().join("etc")).unwrap();
    symlink(elsewhere.path(), linked_folder.path().join("etc/default")).unwrap();

    edit_grub_defaults(linked_folder.path(), &settings()).unwrap_err();

    let linked_file = tempfile::tempdir().unwrap();
    fs::create_dir_all(linked_file.path().join("etc/default")).unwrap();
    symlink(&target, file(linked_file.path())).unwrap();

    edit_grub_defaults(linked_file.path(), &settings()).unwrap_err();
    assert_eq!(fs::read_to_string(&target).unwrap(), BEFORE);
    assert_eq!(fs::read_dir(elsewhere.path()).unwrap().count(), 1);
}

#[test]
fn a_file_that_is_no_text_stays_untouched() {
    let root = system(&[b'G', 0xff, 0xfe, b'\n']);

    edit_grub_defaults(root.path(), &settings()).unwrap_err();
    assert_eq!(
        fs::read(file(root.path())).unwrap(),
        [b'G', 0xff, 0xfe, b'\n']
    );
}
