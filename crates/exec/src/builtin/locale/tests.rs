use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use tempfile::TempDir;
use toolbox_plan::{LocaleEntry, LocaleName};

use super::{SystemLocaleChange, edit_locale_gen, set_system_locale};
use crate::builtin::FileChange;
use crate::testing::mode;

/// `/etc/locale.gen` and `/etc/locale.conf` of the Lima Garuda image,
/// written by cloud-init with only `en_US.UTF-8` enabled.
const LOCALE_GEN: &str = include_str!("../../../../system/src/locales/fixtures/locale.gen");
const LOCALE_CONF: &str = include_str!("../../../../system/src/locales/fixtures/locale.conf");

fn system(locale_gen: Option<&str>) -> (TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let etc = root.path().join("etc");
    fs::create_dir_all(&etc).unwrap();

    if let Some(text) = locale_gen {
        fs::write(etc.join("locale.gen"), text).unwrap();
    }

    (root, etc)
}

fn entry(line: &str) -> LocaleEntry {
    LocaleEntry::parse(line).unwrap()
}

fn locale(name: &str) -> LocaleName {
    LocaleName::new(name).unwrap()
}

/// Edit: the `en_US.UTF-8` line commented out, as a stock glibc file has
/// every line.
fn commented_out() -> String {
    LOCALE_GEN.replace("\nen_US.UTF-8 UTF-8\n", "\n#en_US.UTF-8 UTF-8\n")
}

#[test]
fn enabling_uncomments_the_line_and_keeps_the_rest() {
    let (root, etc) = system(Some(&commented_out()));

    assert_eq!(
        edit_locale_gen(root.path(), &entry("en_US.UTF-8 UTF-8"), true).unwrap(),
        FileChange::Written
    );
    assert_eq!(
        fs::read_to_string(etc.join("locale.gen")).unwrap(),
        LOCALE_GEN
    );
    assert_eq!(mode(&etc.join("locale.gen")), 0o644);
}

/// The file cloud-init writes has no line for other locales at all.
#[test]
fn enabling_a_locale_without_a_line_appends_it() {
    let (root, etc) = system(Some(LOCALE_GEN));

    assert_eq!(
        edit_locale_gen(root.path(), &entry("de_DE.UTF-8 UTF-8"), true).unwrap(),
        FileChange::Written
    );
    assert_eq!(
        fs::read_to_string(etc.join("locale.gen")).unwrap(),
        format!("{LOCALE_GEN}de_DE.UTF-8 UTF-8\n")
    );
}

#[test]
fn disabling_comments_the_line_out() {
    let (root, etc) = system(Some(LOCALE_GEN));

    assert_eq!(
        edit_locale_gen(root.path(), &entry("en_US.UTF-8 UTF-8"), false).unwrap(),
        FileChange::Written
    );
    assert_eq!(
        fs::read_to_string(etc.join("locale.gen")).unwrap(),
        commented_out()
    );
}

#[test]
fn a_file_that_is_as_wanted_is_not_written() {
    let (root, etc) = system(Some(LOCALE_GEN));
    let before = fs::metadata(etc.join("locale.gen"))
        .unwrap()
        .modified()
        .unwrap();

    assert_eq!(
        edit_locale_gen(root.path(), &entry("en_US.UTF-8 UTF-8"), true).unwrap(),
        FileChange::Unchanged
    );
    assert_eq!(
        fs::metadata(etc.join("locale.gen"))
            .unwrap()
            .modified()
            .unwrap(),
        before
    );
}

#[test]
fn a_missing_locale_gen_is_created() {
    let (root, etc) = system(None);

    assert_eq!(
        edit_locale_gen(root.path(), &entry("de_DE.UTF-8 UTF-8"), true).unwrap(),
        FileChange::Written
    );
    assert_eq!(
        fs::read_to_string(etc.join("locale.gen")).unwrap(),
        "de_DE.UTF-8 UTF-8\n"
    );
}

#[test]
fn a_locale_gen_that_is_not_text_is_left_alone() {
    let (root, etc) = system(None);
    fs::write(etc.join("locale.gen"), [0xff_u8, 0xfe_u8]).unwrap();

    edit_locale_gen(root.path(), &entry("de_DE.UTF-8 UTF-8"), true).unwrap_err();
    assert_eq!(
        fs::read(etc.join("locale.gen")).unwrap(),
        [0xff_u8, 0xfe_u8]
    );
}

#[test]
fn a_symlinked_locale_gen_is_refused() {
    let (root, etc) = system(None);
    let elsewhere = root.path().join("elsewhere");
    fs::write(&elsewhere, "untouched\n").unwrap();
    symlink(&elsewhere, etc.join("locale.gen")).unwrap();

    edit_locale_gen(root.path(), &entry("de_DE.UTF-8 UTF-8"), true).unwrap_err();
    assert_eq!(fs::read_to_string(&elsewhere).unwrap(), "untouched\n");
}

/// Edits: `de_DE.UTF-8 UTF-8` appended to `locale.gen`, and an
/// `LC_TIME=de_DE.UTF-8` line appended to `locale.conf`.
#[test]
fn the_system_locale_changes_only_the_lang_line() {
    let (root, etc) = system(Some(&format!("{LOCALE_GEN}de_DE.UTF-8 UTF-8\n")));
    fs::write(
        etc.join("locale.conf"),
        format!("{LOCALE_CONF}LC_TIME=de_DE.UTF-8\n"),
    )
    .unwrap();

    assert_eq!(
        set_system_locale(root.path(), &locale("de_DE.UTF-8")).unwrap(),
        SystemLocaleChange::Changed(FileChange::Written)
    );
    assert_eq!(
        fs::read_to_string(etc.join("locale.conf")).unwrap(),
        "LANG=de_DE.UTF-8\nLC_TIME=de_DE.UTF-8\n"
    );
    assert_eq!(
        set_system_locale(root.path(), &locale("de_DE.UTF-8")).unwrap(),
        SystemLocaleChange::Changed(FileChange::Unchanged)
    );
}

#[test]
fn a_locale_that_is_not_enabled_is_refused_and_nothing_is_written() {
    let (root, etc) = system(Some(LOCALE_GEN));
    fs::write(etc.join("locale.conf"), LOCALE_CONF).unwrap();

    assert_eq!(
        set_system_locale(root.path(), &locale("de_DE.UTF-8")).unwrap(),
        SystemLocaleChange::NotEnabled
    );
    assert_eq!(
        fs::read_to_string(etc.join("locale.conf")).unwrap(),
        LOCALE_CONF
    );
}

#[test]
fn the_c_locale_needs_no_locale_gen() {
    let (root, etc) = system(None);

    assert_eq!(
        set_system_locale(root.path(), &locale("C.UTF-8")).unwrap(),
        SystemLocaleChange::Changed(FileChange::Written)
    );
    assert_eq!(
        fs::read_to_string(etc.join("locale.conf")).unwrap(),
        "LANG=C.UTF-8\n"
    );
}

#[test]
fn a_missing_locale_gen_enables_nothing() {
    let (root, etc) = system(None);

    assert_eq!(
        set_system_locale(root.path(), &locale("en_US.UTF-8")).unwrap(),
        SystemLocaleChange::NotEnabled
    );
    assert!(!etc.join("locale.conf").exists());
}
