use super::{
    CopyKind, InvalidMergeCopy, MergeChoice, MergeCopy, MergeResolve, Resolution, backup_files,
    lists_backup,
};
use crate::testing::merge::{PACMAN_FILES, SUDO_FILES};
use crate::{ConfigFilePath, ConfigText, Fingerprint};

fn copy(path: &str) -> MergeCopy {
    MergeCopy::new(path).unwrap()
}

fn choice(resolution: Resolution, current: bool) -> MergeChoice {
    MergeChoice {
        resolution,
        copy: Fingerprint::of(b"new"),
        current: current.then(|| Fingerprint::of(b"old")),
    }
}

fn merged() -> Resolution {
    Resolution::Merged {
        text: ConfigText::new("merged\n").unwrap(),
    }
}

#[test]
fn each_copy_names_its_kind_and_the_file_it_belongs_to() {
    let cases = [
        ("/etc/pacman.conf.pacnew", CopyKind::New),
        ("/etc/pacman.conf.pacsave", CopyKind::Saved),
        ("/etc/pacman.conf.pacsave.2", CopyKind::OlderSave),
        ("/etc/pacman.conf.pacorig", CopyKind::Orig),
    ];

    for (path, kind) in cases {
        let found = copy(path);
        assert_eq!(found.kind(), kind, "{path}");
        assert_eq!(found.original().as_str(), "/etc/pacman.conf", "{path}");
    }
}

#[test]
fn a_path_that_is_no_copy_is_refused() {
    for path in [
        "/etc/pacman.conf",
        "/etc/pacman.conf.pacsave.",
        "/etc/pacman.conf.pacsave.x",
        "/etc/pacman.conf.bak",
    ] {
        assert!(
            matches!(MergeCopy::new(path), Err(InvalidMergeCopy::NoCopy(_))),
            "{path}"
        );
    }
}

#[test]
fn a_copy_of_a_withheld_file_or_outside_the_config_folders_is_refused() {
    for path in [
        "/etc/shadow.pacnew",
        "/etc/gshadow.pacsave",
        "/usr/lib/os-release.pacnew",
        "/etc/.pacnew",
    ] {
        assert!(
            matches!(MergeCopy::new(path), Err(InvalidMergeCopy::Path(_))),
            "{path}"
        );
    }
}

#[test]
fn a_copy_travels_as_its_path() {
    let found = copy("/etc/mkinitcpio.conf.pacnew");
    let json = serde_json::to_string(&found).unwrap();

    assert_eq!(json, r#""/etc/mkinitcpio.conf.pacnew""#);
    assert_eq!(serde_json::from_str::<MergeCopy>(&json).unwrap(), found);
    serde_json::from_str::<MergeCopy>(r#""/etc/mkinitcpio.conf""#).unwrap_err();
}

#[test]
fn an_older_save_can_only_be_removed() {
    let older = copy("/etc/locale.gen.pacsave.1");

    assert!(choice(Resolution::KeepCurrent, true).fits(&older));
    assert!(!choice(Resolution::TakeCopy, true).fits(&older));
    assert!(!choice(merged(), true).fits(&older));
}

#[test]
fn a_merge_needs_the_file_in_use() {
    let new = copy("/etc/locale.gen.pacnew");

    assert!(choice(merged(), true).fits(&new));
    assert!(!choice(merged(), false).fits(&new));
    assert!(choice(Resolution::TakeCopy, false).fits(&new));
}

#[test]
fn decoding_refuses_a_choice_the_copy_does_not_allow() {
    let resolve = MergeResolve {
        copy: copy("/etc/locale.gen.pacsave.1"),
        choice: choice(Resolution::TakeCopy, true),
    };

    let json = serde_json::to_string(&resolve).unwrap();

    serde_json::from_str::<MergeResolve>(&json).unwrap_err();
}

#[test]
fn a_resolve_survives_the_wire() {
    let resolve = MergeResolve {
        copy: copy("/etc/pacman.conf.pacnew"),
        choice: choice(merged(), true),
    };

    let json = serde_json::to_string(&resolve).unwrap();

    assert_eq!(
        serde_json::from_str::<MergeResolve>(&json).unwrap(),
        resolve
    );
}

#[test]
fn the_console_says_what_happens_to_both_files() {
    let shown = |resolution, current| {
        MergeResolve {
            copy: copy("/etc/pacman.conf.pacnew"),
            choice: choice(resolution, current),
        }
        .command()
    };

    assert_eq!(
        shown(Resolution::KeepCurrent, true),
        "rm /etc/pacman.conf.pacnew"
    );
    assert_eq!(
        shown(Resolution::TakeCopy, false),
        "mv /etc/pacman.conf.pacnew /etc/pacman.conf"
    );
    assert_eq!(
        shown(Resolution::TakeCopy, true),
        "replace /etc/pacman.conf with /etc/pacman.conf.pacnew, the old one kept as /var/lib/garuda-toolbox/backups/etc/pacman.conf"
    );
    assert_eq!(
        shown(merged(), true),
        "write /etc/pacman.conf (7 bytes, merged with /etc/pacman.conf.pacnew), the old one kept as /var/lib/garuda-toolbox/backups/etc/pacman.conf, then rm /etc/pacman.conf.pacnew"
    );
}

#[test]
fn the_backup_files_of_a_package_come_from_its_backup_part() {
    let found: Vec<&str> = backup_files(PACMAN_FILES).collect();

    assert_eq!(
        found,
        [
            "etc/pacman.conf",
            "etc/makepkg.conf",
            "etc/makepkg.conf.d/fortran.conf",
            "etc/makepkg.conf.d/rust.conf",
        ]
    );
}

#[test]
fn a_file_of_the_package_is_no_backup_file() {
    let listed = |path| lists_backup(PACMAN_FILES, &ConfigFilePath::new(path).unwrap());

    assert!(listed("/etc/pacman.conf"));
    assert!(!listed("/etc/pacman.d/mirrorlist"));
    assert!(!lists_backup(
        SUDO_FILES,
        &ConfigFilePath::new("/etc/pacman.conf").unwrap()
    ));
}

#[test]
fn a_list_without_a_backup_part_names_none() {
    let without = PACMAN_FILES.split("%BACKUP%").next().unwrap();

    assert_eq!(backup_files(without).count(), 0);
}
