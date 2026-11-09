use std::collections::BTreeSet;

use toolbox_plan::{NewHome, UserId};

use super::super::{NewAccount, home, new_account, user_id};
use super::snapshot;

fn account<'fields>(
    home: &'fields str,
    uid: &'fields str,
    no_home: bool,
    system: bool,
) -> NewAccount<'fields> {
    NewAccount {
        name: "ada",
        full_name: "",
        shell: "",
        groups: Some(BTreeSet::new()),
        home,
        no_home,
        uid,
        system,
    }
}

#[test]
fn a_typed_user_id_is_a_number_in_range_or_blank() {
    assert_eq!(user_id(""), Ok(None));
    assert_eq!(user_id("  "), Ok(None));
    assert_eq!(
        user_id(" 1500 ").map(|uid| uid.map(UserId::get)),
        Ok(Some(1500))
    );

    for text in [
        "0",
        "-1",
        "60001",
        "1e3",
        "1000.0",
        "abc",
        "99999999999",
        "+5x",
    ] {
        assert_eq!(user_id(text), Err("uid-invalid"), "{text}");
    }
}

#[test]
fn a_typed_home_folder_gets_the_code_of_its_rule() {
    assert_eq!(
        home("/srv/ada").map(|path| path.to_string()),
        Ok("/srv/ada".to_owned())
    );

    for (text, code) in [
        ("", "home-empty"),
        ("srv/ada", "home-not-absolute"),
        ("/srv/../etc", "home-invalid-component"),
        ("/srv/ada/", "home-invalid-component"),
        ("/srv/a:b", "home-invalid-character"),
        ("/srv/ada ", "home-outer-blank"),
        ("/tmp/ada", "home-outside-roots"),
    ] {
        assert_eq!(home(text).err(), Some(code), "{text}");
    }
}

#[test]
fn the_fields_place_the_account_where_the_dialog_says() {
    let snapshot = snapshot();
    let plain = new_account(&account("", "", false, false), &snapshot).unwrap();

    assert_eq!(plain.parts().home, NewHome::Standard);
    assert_eq!(plain.parts().uid, None);
    assert!(!plain.parts().system);

    let placed = new_account(&account("/srv/ada", "1500", false, false), &snapshot).unwrap();

    assert_eq!(placed.parts().home, NewHome::At(home("/srv/ada").unwrap()));
    assert_eq!(placed.parts().uid.map(UserId::get), Some(1500));

    let service = new_account(&account("/srv/ignored", "450", true, true), &snapshot).unwrap();

    assert_eq!(service.parts().home, NewHome::Absent);
    assert!(service.parts().system);
}

#[test]
fn the_fields_refuse_bad_placements_with_a_code() {
    let snapshot = snapshot();

    for (fields, code) in [
        (account("srv", "", false, false), "home-not-absolute"),
        (
            account("/home/../root", "", false, false),
            "home-invalid-component",
        ),
        (account("/home/nico", "", false, false), "home-taken"),
        (account("", "abc", false, false), "uid-invalid"),
        (account("", "0", false, false), "uid-invalid"),
        (account("", "1000", false, false), "uid-in-use"),
        (account("", "450", false, false), "uid-out-of-range"),
        (account("", "1500", false, true), "uid-out-of-range"),
    ] {
        assert_eq!(new_account(&fields, &snapshot).err(), Some(code));
    }
}
