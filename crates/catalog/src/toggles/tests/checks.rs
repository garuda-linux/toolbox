//! One switch per watched state: two entries with the same `check` would
//! show one state in two places.

use super::{SECTION, ToggleCatalog, entry, problem_of};
use crate::Problem;

#[test]
fn checks_are_unique_across_entries() {
    let twice_group = format!(
        "{SECTION}{}{}",
        entry("a", "{ group = \"lp\" }", ""),
        entry("b", "{ group = \"lp\" }", "")
    );

    let twice_package = format!(
        "{SECTION}{}{}",
        entry("a", "{ package = \"cups\" }", ""),
        entry("b", "{ package = \"cups\" }", "")
    );

    let twice_service = format!(
        "{SECTION}{}{}",
        entry("a", "{ service = \"cups.socket\" }", ""),
        entry("b", "{ service = \"cups.socket\" }", "")
    );

    for text in [twice_group, twice_package, twice_service] {
        let (source, id, problem) = problem_of(&text);
        assert_eq!(source, "test.toml", "{text}");

        assert!(
            matches!(problem, Problem::DuplicateCheck(_)),
            "expected a duplicate check for {text:?}, got {problem:?}"
        );
        assert_eq!(id, "b", "{text}");
    }
}

#[test]
fn different_checks_pass() {
    let text = format!(
        "{SECTION}{}{}",
        entry("a", "{ group = \"lp\" }", ""),
        entry("b", "{ group = \"scanner\" }", "")
    );

    ToggleCatalog::parse(&text, "test.toml").unwrap();
}

#[test]
fn a_group_toggle_never_names_an_admin_group() {
    for admins in ["wheel", "sudo"] {
        let text = format!(
            "{SECTION}{}",
            entry("admins", &format!("{{ group = \"{admins}\" }}"), "")
        );

        let (_, id, problem) = problem_of(&text);

        assert_eq!(id, "admins");
        assert_eq!(problem, Problem::AdminGroup(admins.to_owned()));
    }
}
