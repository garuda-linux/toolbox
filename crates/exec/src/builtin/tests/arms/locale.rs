use std::fs;

use toolbox_plan::procedure::Builtin;
use toolbox_plan::protocol::Outcome;
use toolbox_plan::{LocaleEntry, LocaleName};

use super::outcome_of;
use crate::testing::assert_failed;

/// `/etc/locale.gen` of the Lima Garuda image, only `en_US.UTF-8` enabled.
const LOCALE_GEN: &str = include_str!("../../../../../system/src/locales/fixtures/locale.gen");

fn locale_root() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc")).unwrap();
    fs::write(root.path().join("etc/locale.gen"), LOCALE_GEN).unwrap();
    root
}

#[test]
fn switching_a_locale_line_is_one_step() {
    let root = locale_root();
    let builtin = Builtin::EditLocaleGen {
        entry: LocaleEntry::parse("en_US.UTF-8 UTF-8").unwrap(),
        on: false,
    };

    assert_eq!(outcome_of(root.path(), &builtin), Outcome::Succeeded);
    assert_eq!(
        fs::read_to_string(root.path().join("etc/locale.gen")).unwrap(),
        LOCALE_GEN.replace("\nen_US.UTF-8", "\n#en_US.UTF-8")
    );

    let bare = tempfile::tempdir().unwrap();

    assert_failed(outcome_of(bare.path(), &builtin));
}

#[test]
fn the_system_locale_is_one_step() {
    let root = locale_root();
    let builtin = Builtin::SetSystemLocale(LocaleName::new("en_US.UTF-8").unwrap());

    assert_eq!(outcome_of(root.path(), &builtin), Outcome::Succeeded);
    assert_eq!(
        fs::read_to_string(root.path().join("etc/locale.conf")).unwrap(),
        include_str!("../../../../../system/src/locales/fixtures/locale.conf")
    );

    let unwanted = Builtin::SetSystemLocale(LocaleName::new("de_DE.UTF-8").unwrap());

    assert_failed(outcome_of(root.path(), &unwanted));
}
