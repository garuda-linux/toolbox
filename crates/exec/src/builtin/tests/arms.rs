mod locale;

use std::fs;

use toolbox_plan::{
    ConfigFilePath, ConfigText, ConfigWrite, DefaultEntry, Fingerprint, GroupName, GrubSetting,
    GrubSettings, GrubTimeout, HomeDirectory, HomePath, KernelModule, UserConfigEdit, UserName,
};

use super::*;
use crate::testing::assert_failed;

#[test]
fn installing_the_mirrorlist_is_one_step() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc/pacman.d")).unwrap();
    let builtin = Builtin::InstallMirrorlist(mirrorlist("new.example"));
    assert_eq!(outcome_of(root.path(), &builtin), Outcome::Succeeded);
    assert_eq!(
        fs::read_to_string(root.path().join("etc/pacman.d/chaotic-mirrorlist")).unwrap(),
        mirrorlist("new.example").as_str()
    );

    // A second install keeps the first one as the backup.
    let again = execute(root.path(), None, StepId::new(2), &builtin, &mut Vec::new());

    assert_eq!(again, Outcome::Succeeded);
    assert_eq!(
        fs::read_to_string(root.path().join("etc/pacman.d/chaotic-mirrorlist.bak")).unwrap(),
        mirrorlist("new.example").as_str()
    );
}

#[test]
fn a_missing_mirrorlist_folder_fails_the_step() {
    let root = tempfile::tempdir().unwrap();
    let builtin = Builtin::InstallMirrorlist(mirrorlist("new.example"));

    assert_failed(outcome_of(root.path(), &builtin));
}

#[test]
fn accounts_and_homes_are_checked_as_steps() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc")).unwrap();
    fs::write(
        root.path().join("etc/passwd"),
        "nico:x:1000:1000::/home/nico:/usr/bin/bash\n",
    )
    .unwrap();
    fs::write(
        root.path().join("etc/group"),
        "nico:x:1000:\nmakers:x:1001:nico\n",
    )
    .unwrap();

    let ordinary = Builtin::RequireOrdinaryUser(UserName::new("nico").unwrap());

    assert_eq!(outcome_of(root.path(), &ordinary), Outcome::Succeeded);

    let member = Builtin::RequireUserGroup(GroupName::new("makers").unwrap());

    assert_eq!(outcome_of(root.path(), &member), Outcome::Succeeded);

    let free = Builtin::RequireFreeHome(HomeDirectory::new("/home/fresh").unwrap());

    assert_eq!(outcome_of(root.path(), &free), Outcome::Succeeded);
}

#[test]
fn a_user_file_is_not_the_helpers_to_change() {
    let edit = Builtin::EditUserConfig {
        edit: UserConfigEdit::File {
            path: HomePath::new(".config/app.conf").unwrap(),
            content: "on = true\n".to_owned(),
        },
        on: true,
    };

    assert_failed(outcome_of(tempfile::tempdir().unwrap().path(), &edit));
}

#[test]
fn blacklisting_a_module_is_one_step() {
    let root = tempfile::tempdir().unwrap();
    let builtin = Builtin::EditModprobeBlacklist {
        module: KernelModule::new("nouveau").unwrap(),
        on: true,
    };

    assert_eq!(outcome_of(root.path(), &builtin), Outcome::Succeeded);
    assert!(
        fs::read_to_string(
            root.path()
                .join("etc/modprobe.d/garuda-toolbox-blacklist.conf")
        )
        .unwrap()
        .contains("blacklist nouveau")
    );

    let blocked = tempfile::tempdir().unwrap();
    fs::create_dir_all(blocked.path().join("etc")).unwrap();
    fs::write(blocked.path().join("etc/modprobe.d"), "").unwrap();
    let blocked_builtin = Builtin::EditModprobeBlacklist {
        module: KernelModule::new("nouveau").unwrap(),
        on: false,
    };

    assert_failed(outcome_of(blocked.path(), &blocked_builtin));
}

#[test]
fn grub_settings_are_one_step() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc/default")).unwrap();
    fs::write(root.path().join("etc/default/grub"), "GRUB_TIMEOUT=5\n").unwrap();
    let builtin = Builtin::EditGrubDefaults(
        GrubSettings::new(vec![
            GrubSetting::Default(DefaultEntry::Saved),
            GrubSetting::Timeout(GrubTimeout::new(10).unwrap()),
        ])
        .unwrap(),
    );

    assert_eq!(outcome_of(root.path(), &builtin), Outcome::Succeeded);
    assert!(
        fs::read_to_string(root.path().join("etc/default/grub"))
            .unwrap()
            .contains("GRUB_TIMEOUT=10")
    );
}

#[test]
fn writing_a_config_file_is_one_step() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc")).unwrap();
    fs::write(root.path().join("etc/fstab"), "old = 1\n").unwrap();
    let builtin = Builtin::WriteConfigFile(ConfigWrite {
        path: ConfigFilePath::new("/etc/fstab").unwrap(),
        content: ConfigText::new("new = 2\n").unwrap(),
        original: Fingerprint::of(b"old = 1\n"),
    });

    assert_eq!(outcome_of(root.path(), &builtin), Outcome::Succeeded);
    assert_eq!(
        fs::read_to_string(root.path().join("etc/fstab")).unwrap(),
        "new = 2\n"
    );
}

#[test]
fn an_unreadable_lock_file_fails_the_lock_step() {
    let dir = tempfile::tempdir().unwrap();
    let blocker = dir.path().join("file");
    fs::write(&blocker, "").unwrap();

    assert!(
        remove_database_lock(dir.path(), &blocker.join("db.lck")).is_err(),
        "a lock below a file is no lock at all"
    );
}

#[test]
fn a_process_without_readable_descriptors_is_no_holder() {
    let dir = tempfile::tempdir().unwrap();
    let proc_root = dir.path().join("proc");
    let lock = dir.path().join("db.lck");

    fs::write(&lock, "").unwrap();
    fs::create_dir_all(proc_root.join("5")).unwrap();

    assert_eq!(
        remove_database_lock(&proc_root, &lock).unwrap(),
        LockRemoval::Removed
    );
}

#[test]
fn a_held_lock_fails_its_step_with_the_holder() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root");
    fs::create_dir_all(root.join("proc/777")).unwrap();
    fs::create_dir_all(root.join("var/lib/pacman")).unwrap();
    let lock = root.join("var/lib/pacman/db.lck");
    fs::write(&lock, "").unwrap();
    fs::write(root.join("proc/777/cmdline"), "pacman").unwrap();

    let held = lock_holder(&root.join("proc"), &fs::metadata(&lock).unwrap()).unwrap();

    assert_eq!(held, None, "a cmdline is no open file");

    process(&root.join("proc"), "778", &[&lock]);

    assert_failed(outcome_of(&root, &Builtin::RemoveDatabaseLock));
    assert!(lock.exists());
}

#[test]
fn an_uncheckable_home_fails_its_step() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("etc"), "").unwrap();
    let builtin = Builtin::RequireFreeHome(HomeDirectory::new("/etc/taken").unwrap());

    assert_failed(outcome_of(root.path(), &builtin));
}

#[test]
fn taking_a_module_off_a_missing_folder_changes_nothing() {
    let root = tempfile::tempdir().unwrap();
    let builtin = Builtin::EditModprobeBlacklist {
        module: KernelModule::new("nouveau").unwrap(),
        on: false,
    };

    assert_eq!(outcome_of(root.path(), &builtin), Outcome::Skipped);
}
