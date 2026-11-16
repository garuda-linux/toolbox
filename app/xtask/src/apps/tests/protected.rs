//! The protection check of every package of an app.

use super::super::{CATEGORIES, Lists};
use super::app;

#[test]
fn a_protected_package_after_the_first_is_checked() {
    let lists = Lists::new(vec![(
        CATEGORIES[5],
        vec![app("Logins", &["doas", "util-linux"])],
    )])
    .without_commands_of();

    let names: Vec<_> = lists.packages().into_iter().collect();
    assert!(names.is_empty(), "{names:?}");
    assert_eq!(
        lists.dropped(),
        ["security: \"Logins\" is a command of the protected util-linux"]
    );
}
