use std::fs;
use std::os::unix::fs::{PermissionsExt as _, symlink};
use std::path::Path;

use toolbox_plan::{ConfigText, EditCopy};

use super::EditFolder;

fn mode(path: &Path) -> u32 {
    fs::metadata(path).unwrap().permissions().mode() & 0o7777
}

#[test]
fn a_copy_is_private_and_the_only_one() {
    let base = tempfile::tempdir().unwrap();
    let folder = EditFolder::new(base.path());
    let text = ConfigText::new("a = 1\n").unwrap();
    let first = folder.create("fstab", &text).unwrap();

    assert_eq!(
        first.as_str(),
        base.path()
            .join("garuda-toolbox/edit/fstab")
            .to_str()
            .unwrap()
    );
    assert_eq!(mode(folder.path()), 0o700);
    assert_eq!(mode(Path::new(first.as_str())), 0o600);
    assert_eq!(folder.read(&first).unwrap().unwrap(), text);

    let second = folder
        .create("grub", &ConfigText::new("b\n").unwrap())
        .unwrap();

    assert!(!Path::new(first.as_str()).exists(), "the old copy stayed");
    assert_eq!(folder.read(&second).unwrap().unwrap().as_str(), "b\n");

    folder.remove(&second).unwrap();
    folder.remove(&second).unwrap();
    assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 0);
}

#[test]
fn a_wide_open_edit_folder_is_made_private_again() {
    let base = tempfile::tempdir().unwrap();
    let folder = EditFolder::new(base.path());

    fs::create_dir_all(folder.path()).unwrap();
    fs::set_permissions(folder.path(), fs::Permissions::from_mode(0o777)).unwrap();
    folder
        .create("fstab", &ConfigText::new("a\n").unwrap())
        .unwrap();

    assert_eq!(mode(folder.path()), 0o700);
}

#[test]
fn a_linked_edit_folder_is_refused() {
    let base = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    let folder = EditFolder::new(base.path());

    fs::create_dir_all(base.path().join("garuda-toolbox")).unwrap();
    symlink(elsewhere.path(), folder.path()).unwrap();

    folder
        .create("sudoers", &ConfigText::new("secret\n").unwrap())
        .unwrap_err();
    assert_eq!(fs::read_dir(elsewhere.path()).unwrap().count(), 0);
}

#[test]
fn what_the_editor_left_is_read_back_or_refused() {
    let base = tempfile::tempdir().unwrap();
    let folder = EditFolder::new(base.path());
    let copy = folder
        .create("fstab", &ConfigText::new("a\n").unwrap())
        .unwrap();

    folder.stand_in_edit(&copy).unwrap();

    let edited = folder.read(&copy).unwrap().unwrap();

    assert!(edited.as_str().starts_with("a\n# Edited by the dry run"));
    assert!(edited.as_str().ends_with('\n'));

    fs::write(copy.as_str(), [b'a', 0, b'b']).unwrap();
    folder.read(&copy).unwrap().unwrap_err();

    fs::remove_file(copy.as_str()).unwrap();
    folder.read(&copy).unwrap_err();
}

#[test]
fn a_copy_of_another_folder_is_not_touched() {
    let base = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    let foreign = EditFolder::new(other.path())
        .create("fstab", &ConfigText::new("theirs\n").unwrap())
        .unwrap();

    let folder = EditFolder::new(base.path());

    folder.read(&foreign).unwrap_err();
    folder.remove(&foreign).unwrap_err();
    folder.stand_in_edit(&foreign).unwrap_err();
    assert_eq!(fs::read_to_string(foreign.as_str()).unwrap(), "theirs\n");
}

#[test]
fn the_edit_folder_prefers_the_runtime_folder() {
    let runtime = tempfile::tempdir().unwrap();
    let cache = Path::new("/home/user/.cache/garuda-toolbox");
    let with = |value: Option<&str>| {
        let value = value.map(std::ffi::OsString::from);

        EditFolder::from_env(
            |key| {
                assert_eq!(key, "XDG_RUNTIME_DIR");
                value.clone()
            },
            cache,
        )
    };

    assert_eq!(
        with(runtime.path().to_str()).path(),
        runtime.path().join("garuda-toolbox/edit")
    );

    for unusable in [None, Some("relative/run"), Some("/no/such/runtime/folder")] {
        assert_eq!(with(unusable).path(), cache.join("edit"), "{unusable:?}");
    }

    EditCopy::new(format!("{}/fstab", with(None).path().display())).unwrap();
}
