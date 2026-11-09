mod checks;
mod components;
mod edits;
mod extras;
mod gaming;
mod texts;
mod user_config;

use std::fs;

use toolbox_plan::{GroupName, PackageName, UnitName};

use super::{Check, Requires, ToggleCatalog};
use crate::{CatalogError, Problem};

const SECTION: &str = "[[section]]\nid = \"containers\"\ntitle = \"Containers\"\nicon = \"x\"\n";

fn entry(id: &str, check: &str, requires: &str) -> String {
    let requires = if requires.is_empty() {
        String::new()
    } else {
        format!("requires = {requires}\n")
    };

    format!(
        "[[section.entry]]\nid = \"{id}\"\ntitle = \"T\"\ndescription = \"D\"\ncheck = {check}\n{requires}"
    )
}

fn package(id: &str, requires: &str) -> String {
    entry(id, &format!("{{ package = \"{id}\" }}"), requires)
}

fn problem_of(text: &str) -> (String, String, Problem) {
    match ToggleCatalog::parse(text, "test.toml") {
        Err(CatalogError::Invalid {
            source_name,
            id,
            problem,
        }) => (source_name, id, problem),
        other => panic!("expected a validation error, got {other:?}"),
    }
}

#[test]
fn parses_sections_entries_and_every_kind_of_check() {
    let text = format!(
        "{SECTION}{}{}{}{}",
        package("docker", ""),
        entry(
            "enabled-docker",
            "{ service = \"docker.service\" }",
            "{ all = [\"docker\"] }"
        ),
        entry("user-unit", "{ user-service = \"pipewire.socket\" }", ""),
        entry(
            "group-docker",
            "{ group = \"docker\" }",
            "{ all = [\"docker\"] }"
        ),
    );

    let catalog = ToggleCatalog::parse(&text, "test.toml").unwrap();
    let section = &catalog.sections()[0];

    assert_eq!(section.id(), "containers");
    assert_eq!(section.title(), "Containers");
    assert_eq!(section.icon(), "x");

    let checks: Vec<&Check> = section
        .entries()
        .iter()
        .map(super::ToggleEntry::check)
        .collect();

    assert_eq!(
        checks,
        [
            &Check::Package(PackageName::new("docker").unwrap()),
            &Check::Service(UnitName::new("docker.service").unwrap()),
            &Check::UserService(UnitName::new("pipewire.socket").unwrap()),
            &Check::Group(GroupName::new("docker").unwrap()),
        ]
    );

    let enabled = catalog.entry("enabled-docker").unwrap();

    assert_eq!(enabled.id(), "enabled-docker");
    assert_eq!(enabled.title(), "T");
    assert_eq!(enabled.description(), "D");
    assert_eq!(
        enabled.requires(),
        &Requires::All(vec!["docker".to_owned()])
    );
    assert!(catalog.entry("containers").is_none());
    assert!(catalog.entry("podman").is_none());
}

#[test]
fn requires_is_optional() {
    let text = format!(
        "{SECTION}[[section.entry]]\nid = \"a\"\ntitle = \"A\"\ndescription = \"D\"\ncheck = {{ package = \"a\" }}\n"
    );

    let catalog = ToggleCatalog::parse(&text, "test.toml").unwrap();

    assert_eq!(catalog.entry("a").unwrap().requires(), &Requires::Nothing);
}

#[test]
fn an_empty_file_is_an_empty_catalog() {
    assert_eq!(
        ToggleCatalog::parse("", "test.toml").unwrap(),
        ToggleCatalog::default()
    );
}

#[test]
fn syntax_errors_unknown_fields_and_bad_checks_name_the_file() {
    for text in [
        "[[section]\n".to_owned(),
        format!("{SECTION}extra = 1\n{}", package("a", "")),
        format!("{SECTION}{}exclusive = []\n", package("a", "")),
        format!("{SECTION}{}", entry("a", "{ toggle = \"a\" }", "")),
        format!("{SECTION}{}", entry("a", "{}", "")),
        format!(
            "{SECTION}{}",
            entry("a", "{ package = \"a\", group = \"a\" }", "")
        ),
        "[[section]]\nid = \"s\"\ntitle = \"S\"\n".to_owned(),
    ] {
        match ToggleCatalog::parse(&text, "test.toml") {
            Err(
                error @ CatalogError::Syntax {
                    source_name: _,
                    message: _,
                },
            ) => {
                assert!(error.to_string().starts_with("test.toml: "), "{error}");
            }
            other => panic!("expected a syntax error for {text:?}, got {other:?}"),
        }
    }
}

#[test]
fn rejects_bad_section_and_entry_ids() {
    let section = "[[section]]\nid = \"Audio\"\ntitle = \"A\"\nicon = \"x\"\n";

    assert_eq!(
        problem_of(&format!("{section}{}", package("a", ""))),
        ("test.toml".to_owned(), "Audio".to_owned(), Problem::BadId)
    );
    assert_eq!(
        problem_of(&format!("{SECTION}{}", package("network_manager", ""))).2,
        Problem::BadId
    );
}

#[test]
fn ids_are_unique_across_sections_and_entries() {
    let twice = format!("{SECTION}{}{}", package("a", ""), package("a", ""));
    let other_section = format!("{SECTION}{}{SECTION}{}", package("a", ""), package("b", ""));

    let entry_named_like_section = format!("{SECTION}{}", package("containers", ""));

    for text in [twice, other_section, entry_named_like_section] {
        assert_eq!(problem_of(&text).2, Problem::DuplicateId, "{text}");
    }
}

#[test]
fn rejects_empty_texts() {
    let blank_title = "[[section]]\nid = \"s\"\ntitle = \" \"\nicon = \"x\"\n";
    let blank_icon = "[[section]]\nid = \"s\"\ntitle = \"S\"\nicon = \"\"\n";
    let entry_title = "[[section.entry]]\nid = \"a\"\ntitle = \"\"\ndescription = \"D\"\ncheck = { package = \"a\" }\n";
    let entry_description = "[[section.entry]]\nid = \"a\"\ntitle = \"T\"\ndescription = \"\\n\"\ncheck = { package = \"a\" }\n";

    assert_eq!(
        problem_of(&format!("{blank_title}{}", package("a", ""))).2,
        Problem::EmptyTitle
    );
    assert_eq!(
        problem_of(&format!("{blank_icon}{}", package("a", ""))).2,
        Problem::EmptyIcon
    );
    assert_eq!(
        problem_of(&format!("{SECTION}{entry_title}")),
        ("test.toml".to_owned(), "a".to_owned(), Problem::EmptyTitle)
    );
    assert_eq!(
        problem_of(&format!("{SECTION}{entry_description}")).2,
        Problem::EmptyDescription
    );
}

#[test]
fn rejects_a_section_without_entries() {
    assert_eq!(
        problem_of(SECTION),
        (
            "test.toml".to_owned(),
            "containers".to_owned(),
            Problem::NoEntries
        )
    );
}

#[test]
fn rejects_invalid_names_with_the_plan_rule() {
    for (check, expected) in [
        (
            "{ package = \"-rf\" }",
            PackageName::new("-rf").unwrap_err(),
        ),
        (
            "{ service = \"docker\" }",
            UnitName::new("docker").unwrap_err(),
        ),
        (
            "{ user-service = \"a b.service\" }",
            UnitName::new("a b.service").unwrap_err(),
        ),
        ("{ group = \"\" }", GroupName::new("").unwrap_err()),
    ] {
        let text = format!("{SECTION}{}", entry("a", check, ""));

        assert_eq!(
            problem_of(&text),
            (
                "test.toml".to_owned(),
                "a".to_owned(),
                Problem::BadName(expected)
            )
        );
    }
}

#[test]
fn requires_must_name_another_entry_of_the_file() {
    let unknown = format!("{SECTION}{}", package("a", "{ all = [\"b\"] }"));
    let section = format!("{SECTION}{}", package("a", "{ all = [\"containers\"] }"));
    let itself = format!("{SECTION}{}", package("a", "{ all = [\"a\"] }"));

    assert_eq!(
        problem_of(&unknown),
        (
            "test.toml".to_owned(),
            "a".to_owned(),
            Problem::UnknownRequirement("b".to_owned())
        )
    );
    assert_eq!(
        problem_of(&section).2,
        Problem::UnknownRequirement("containers".to_owned())
    );
    assert_eq!(problem_of(&itself).2, Problem::RequiresItself);
}

#[test]
fn requires_names_no_entry_twice() {
    let text = format!(
        "{SECTION}{}{}",
        package("a", ""),
        package("b", "{ all = [\"a\", \"a\"] }")
    );

    assert_eq!(
        problem_of(&text),
        (
            "test.toml".to_owned(),
            "b".to_owned(),
            Problem::RequirementRepeats("a".to_owned())
        )
    );
}

#[test]
fn requires_may_name_an_entry_of_another_section_or_later_in_the_file() {
    let text = format!(
        "{SECTION}{}[[section]]\nid = \"other\"\ntitle = \"O\"\nicon = \"x\"\n{}",
        package("a", "{ all = [\"b\"] }"),
        package("b", "")
    );

    ToggleCatalog::parse(&text, "test.toml").unwrap();
}

#[test]
fn rejects_requirement_cycles() {
    let text = format!(
        "{SECTION}{}{}{}{}",
        package("a", ""),
        package("b", "{ all = [\"a\", \"c\"] }"),
        package("c", "{ all = [\"d\"] }"),
        package("d", "{ all = [\"b\"] }"),
    );

    let (_, id, problem) = problem_of(&text);

    assert_eq!(id, "b");
    assert_eq!(
        problem,
        Problem::RequirementCycle(vec!["b".to_owned(), "c".to_owned(), "d".to_owned()])
    );
    assert_eq!(
        problem.to_string(),
        "the requirements form a cycle: b -> c -> d -> b"
    );
}

#[test]
fn shared_requirements_are_no_cycle() {
    let text = format!(
        "{SECTION}{}{}{}{}",
        package("a", ""),
        package("b", "{ all = [\"a\"] }"),
        package("c", "{ all = [\"a\", \"b\"] }"),
        package("d", "{ all = [\"c\", \"b\"] }"),
    );

    ToggleCatalog::parse(&text, "test.toml").unwrap();
}

#[test]
fn load_reads_the_named_file_and_errors_name_it() {
    let dir = tempfile::tempdir().unwrap();
    let missing = ToggleCatalog::load(dir.path(), "toggles.toml").unwrap_err();

    assert!(missing.to_string().contains("toggles.toml"), "{missing}");

    fs::write(
        dir.path().join("toggles.toml"),
        format!("{SECTION}{}", package("a", "")),
    )
    .unwrap();

    let catalog = ToggleCatalog::load(dir.path(), "toggles.toml").unwrap();

    assert_eq!(catalog.sections().len(), 1);
}

#[test]
fn requires_says_whether_all_or_any_entry_is_needed() {
    let text = format!(
        "{SECTION}{}{}{}",
        package("docker", ""),
        package("podman", ""),
        package("compose", "{ any = [\"docker\", \"podman\"] }"),
    );

    let catalog = ToggleCatalog::parse(&text, "test.toml").unwrap();
    let requires = catalog.entry("compose").unwrap().requires();

    assert_eq!(requires.ids(), ["docker".to_owned(), "podman".to_owned()]);
    assert!(requires.is_met(|id| id == "podman"));
    assert!(
        !Requires::All(vec!["docker".to_owned(), "podman".to_owned()]).is_met(|id| id == "podman")
    );
    assert!(Requires::Nothing.is_met(|_| false));
}

#[test]
fn a_requires_list_must_name_something_and_use_all_or_any() {
    let empty = format!(
        "{SECTION}{}{}",
        package("a", ""),
        package("b", "{ any = [] }")
    );

    let bare_list = format!("{SECTION}{}{}", package("a", ""), package("b", "[\"a\"]"));
    let both = format!(
        "{SECTION}{}{}",
        package("a", ""),
        package("b", "{ all = [\"a\"], any = [\"a\"] }")
    );

    assert_eq!(problem_of(&empty).2, Problem::EmptyRequirement);
    assert!(matches!(
        ToggleCatalog::parse(&bare_list, "test.toml"),
        Err(CatalogError::Syntax {
            source_name: _,
            message: _
        })
    ));
    assert!(matches!(
        ToggleCatalog::parse(&both, "test.toml"),
        Err(CatalogError::Syntax {
            source_name: _,
            message: _
        })
    ));
}
