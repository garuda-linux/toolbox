use std::collections::BTreeMap;
use std::fs;

use toolbox_plan::{ConfigWord, HomePath, UserConfigEdit};
use toolbox_state::parts::UserConfigStates;

use super::{HomeConfigs, UserConfigSource as _};
use crate::ProbeError;

fn kwin() -> UserConfigEdit {
    UserConfigEdit::IniKey {
        path: HomePath::new(".config/kwinrc").unwrap(),
        group: ConfigWord::new("Windows").unwrap(),
        key: ConfigWord::new("BorderlessMaximizedWindows").unwrap(),
        on: ConfigWord::new("true").unwrap(),
        off: ConfigWord::new("false").unwrap(),
    }
}

fn audio() -> UserConfigEdit {
    UserConfigEdit::File {
        path: HomePath::new(".config/wireplumber/wireplumber.conf.d/x.conf").unwrap(),
        content: "x\n".to_owned(),
    }
}

fn unreadable() -> UserConfigEdit {
    UserConfigEdit::JsonFlag {
        path: HomePath::new(".config/folder").unwrap(),
        key: ConfigWord::new("KEY").unwrap(),
    }
}

#[test]
fn reads_each_edit_and_leaves_out_what_cannot_be_read() {
    let home = tempfile::tempdir().unwrap();
    fs::create_dir_all(home.path().join(".config/folder")).unwrap();
    fs::write(
        home.path().join(".config/kwinrc"),
        "[Windows]\nBorderlessMaximizedWindows=true\n",
    )
    .unwrap();

    let states = HomeConfigs::new(home.path())
        .states(&[kwin(), audio(), unreadable()])
        .unwrap();

    assert_eq!(
        states,
        UserConfigStates(BTreeMap::from([(kwin(), true), (audio(), false)]))
    );
}

#[test]
fn without_a_home_only_an_empty_list_has_an_answer() {
    let homeless = HomeConfigs { home: None };

    assert_eq!(homeless.states(&[]), Ok(UserConfigStates::default()));
    assert_eq!(homeless.states(&[kwin()]), Err(ProbeError::Unavailable));
}
