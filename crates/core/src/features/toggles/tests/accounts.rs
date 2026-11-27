//! A group toggle meets the guards of the Users page, and a refused click
//! says why.

use toolbox_catalog::ToggleCatalog;

use super::super::{Refusal, toggle};
use super::snapshot;
use crate::features::{RefusalText as _, users};
use crate::test_support::membership;
use crate::{ChangeSet, Toggle};

const OWN_GROUP: &str = r#"
[[section]]
id = "groups"
title = "Groups"
icon = "x"

[[section.entry]]
id = "own-group"
title = "Own group"
description = "The primary group of the test user."
check = { group = "ada" }

[[section.entry]]
id = "cups"
title = "CUPS"
description = "C"
check = { package = "cups" }

[[section.entry]]
id = "printing"
title = "Printing"
description = "P"
check = { group = "lp" }
requires = { all = ["cups"] }
"#;

fn catalog() -> ToggleCatalog {
    ToggleCatalog::parse(OWN_GROUP, "test.toml").unwrap()
}

#[test]
fn a_group_toggle_never_leaves_the_primary_group() {
    let system = snapshot(&[], &[], &[]);
    let mut changes = ChangeSet::default();

    let refused = toggle(&catalog(), "own-group", Toggle::Off, &system, &mut changes).unwrap_err();

    assert_eq!(
        refused.refusal,
        Refusal::Account(users::Refusal::PrimaryGroup)
    );
    assert_eq!(refused.refusal.code(), "primary-group");
    assert_eq!(refused.subject, "ada", "the text names the group");
    assert!(changes.is_empty(), "{changes:?}");
}

#[test]
fn an_account_refusal_names_the_user_unless_it_is_about_the_group() {
    let membership = membership("grace", "lp");
    let last_admin = Refusal::of_membership(users::Refusal::LastAdmin, &membership);
    let primary = Refusal::of_membership(users::Refusal::PrimaryGroup, &membership);

    assert_eq!(last_admin.subject, "grace");
    assert_eq!(primary.subject, "lp");
}

#[test]
fn an_account_refusal_shows_the_users_page_text() {
    assert_eq!(
        Refusal::Account(users::Refusal::LastAdmin).text_id(),
        "account-refusal-last-admin"
    );
}

#[test]
fn unknown_and_waiting_entries_say_so() {
    let system = snapshot(&[], &[], &[]);
    let mut changes = ChangeSet::default();

    let unknown = toggle(&catalog(), "nope", Toggle::On, &system, &mut changes).unwrap_err();
    let waiting = toggle(&catalog(), "printing", Toggle::On, &system, &mut changes).unwrap_err();

    assert_eq!(unknown.refusal.code(), "unknown");
    assert_eq!(waiting.refusal.code(), "unmet");
    assert!(changes.is_empty(), "{changes:?}");
}
