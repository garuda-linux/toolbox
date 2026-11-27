mod placement;

use std::collections::BTreeSet;

use toolbox_plan::{GroupName, ShellPath, UserName};

use super::{InputProblem, NewAccount, login_from_full_name, name, new_account};
use crate::Snapshot;
use crate::domains::{Accounts, AvailableShells};
use crate::features::RefusalText;
use crate::features::users::Refusal;
use crate::test_support::{group, group_name, user};

fn filled<'fields>(
    name: &'fields str,
    full: &'fields str,
    shell: &'fields str,
    groups: Option<BTreeSet<GroupName>>,
) -> NewAccount<'fields> {
    NewAccount {
        name,
        full_name: full,
        shell,
        groups,
        home: "",
        no_home: false,
        uid: "",
        system: false,
    }
}

fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(Accounts {
        users: vec![user("ada", 1000, 1000)],
        groups: vec![group("wheel", 998, &[])],
    });
    snapshot.insert(AvailableShells(vec![
        ShellPath::new("/usr/bin/bash").unwrap(),
    ]));
    snapshot
}

#[test]
fn typed_names_get_the_code_of_the_rule_they_break() {
    for (text, code) in [
        ("", "empty"),
        ("Grace", "invalid-start"),
        ("1ada", "invalid-start"),
        ("-grace", "invalid-start"),
        ("grace lovelace", "invalid-character"),
        ("grace:x", "invalid-character"),
        ("adä", "invalid-character"),
        ("a2345678901234567890123456789012x", "too-long"),
    ] {
        let checked: Result<UserName, _> = name(text);

        assert_eq!(checked.err().map(RefusalText::code), Some(code), "{text}");
    }

    let user: Result<UserName, _> = name("ada_l-2");
    let group: Result<GroupName, _> = name("_makers");

    assert!(user.is_ok() && group.is_ok());
}

#[test]
fn the_add_user_fields_are_checked_field_by_field() {
    let snapshot = snapshot();
    let empty: Option<BTreeSet<GroupName>> = Some(BTreeSet::new());

    for (input, code) in [
        (filled("", "", "", empty.clone()), "empty"),
        (filled("Grace", "", "", empty.clone()), "invalid-start"),
        (filled("ada", "", "", empty.clone()), "user-exists"),
        (filled("wheel", "", "", empty.clone()), "group-exists"),
        (
            filled("grace", "a:b", "", empty.clone()),
            "invalid-character",
        ),
        (filled("grace", " Grace", "", empty.clone()), "outer-blank"),
        (
            filled("grace", "", "bash", empty.clone()),
            "shell-not-listed",
        ),
        (
            filled("grace", "", "/usr/bin/zsh", empty),
            "shell-not-listed",
        ),
        (
            filled(
                "grace",
                "",
                "",
                Some(BTreeSet::from([group_name("ghosts")])),
            ),
            "unknown-group",
        ),
        (filled("grace", "", "", None), "invalid"),
    ] {
        assert_eq!(
            new_account(&input, &snapshot).err().map(RefusalText::code),
            Some(code)
        );
    }
}

#[test]
fn a_checked_add_user_form_builds_the_user() {
    let snapshot = snapshot();
    let member = Some(BTreeSet::from([group_name("wheel")]));
    let user = new_account(
        &NewAccount {
            name: "grace",
            full_name: "Grace Lovelace",
            shell: "/usr/bin/bash",
            groups: member,
            home: "",
            no_home: false,
            uid: "",
            system: false,
        },
        &snapshot,
    )
    .unwrap();

    assert_eq!(user.parts().name.as_str(), "grace");
    assert_eq!(user.parts().full_name.as_str(), "Grace Lovelace");
    assert_eq!(
        user.parts().shell.as_ref().map(ShellPath::as_str),
        Some("/usr/bin/bash")
    );
    assert_eq!(user.parts().groups.len(), 1);
    assert_eq!(
        new_account(&filled("grace", "", "", Some(BTreeSet::new())), &snapshot)
            .map(|created| created.into_parts().shell),
        Ok(None)
    );
    assert_eq!(
        new_account(
            &filled("grace", "", "", Some(BTreeSet::new())),
            &Snapshot::default()
        )
        .err(),
        Some(InputProblem::Refused(Refusal::Unavailable))
    );
}

#[test]
fn the_add_button_names_the_first_thing_that_holds_it() {
    use super::{AddBlocker, AddForm, add_blocker};

    fn form<'fields>(
        name: &'fields str,
        name_ok: bool,
        first: &'fields str,
        second: &'fields str,
    ) -> AddForm<'fields> {
        AddForm {
            name,
            name_ok,
            first,
            second,
            password_ok: !first.is_empty() && first == second,
            rest_ok: true,
        }
    }

    assert_eq!(
        add_blocker(&form("", false, "", "")),
        Some(AddBlocker::NoName)
    );
    assert_eq!(
        add_blocker(&form("Grace!", false, "", "")),
        Some(AddBlocker::BadName)
    );
    assert_eq!(
        add_blocker(&form("grace", true, "", "")),
        Some(AddBlocker::NoPassword)
    );
    assert_eq!(
        add_blocker(&form("grace", true, "one", "")),
        Some(AddBlocker::NoRepeat)
    );
    assert_eq!(
        add_blocker(&form("grace", true, "one", "two")),
        Some(AddBlocker::Fields)
    );
    assert_eq!(add_blocker(&form("grace", true, "one", "one")), None);

    let rest = AddForm {
        rest_ok: false,
        ..form("grace", true, "one", "one")
    };

    assert_eq!(add_blocker(&rest), Some(AddBlocker::Fields));

    let weak = AddForm {
        password_ok: false,
        ..form("grace", true, "one", "one")
    };

    assert_eq!(add_blocker(&weak), Some(AddBlocker::Fields));
}

/// The add-user dialog asks the full name first and proposes the login
/// from its first word that makes one.
#[test]
fn the_login_follows_the_first_word_of_the_full_name() {
    for (full_name, login) in [
        ("Grace Lovelace", "grace"),
        ("  Grace   Hopper", "grace"),
        ("Jürgen Groß", "juergen"),
        ("Émile Zola", "emile"),
        ("Øystein Ærø", "oystein"),
        ("42 Douglas", "douglas"),
        ("Mary-Kate O'Neil", "mary-kate"),
        ("-x", "x"),
        ("李 Lee", "lee"),
        ("", ""),
        ("   ", ""),
    ] {
        assert_eq!(login_from_full_name(full_name), login, "{full_name}");
    }
}

#[test]
fn a_proposed_login_is_always_a_valid_user_name() {
    let long = "A".repeat(40);

    let login = login_from_full_name(&long);

    assert_eq!(login.chars().count(), 32);
    UserName::new(login).unwrap();
}
