use std::path::Path;

use toolbox_plan::{PackageName, UnitName};

use super::{SECTION, entry, problem_of};
use crate::{CatalogError, Problem, ToggleCatalog};

fn gamemode(extra: &str) -> String {
    format!(
        "{SECTION}{}{extra}",
        entry("gamemode", "{ package = \"gamemode\" }", "")
    )
}

#[test]
fn also_lists_packages_that_come_with_the_entry() {
    let text = gamemode("also = [{ package = \"lib32-gamemode\" }]\n");
    let catalog = ToggleCatalog::parse(&text, "test.toml").unwrap();

    assert_eq!(
        catalog.entry("gamemode").unwrap().also(),
        [PackageName::new("lib32-gamemode").unwrap()]
    );
}

#[test]
fn without_also_nothing_comes_along() {
    let catalog = ToggleCatalog::parse(&gamemode(""), "test.toml").unwrap();
    let entry = catalog.entry("gamemode").unwrap();

    assert_eq!(entry.also(), []);
    assert_eq!(entry.icon(), None);
}

#[test]
fn also_needs_a_package_check_and_new_valid_packages() {
    let on_a_service = format!(
        "{SECTION}{}also = [{{ package = \"b\" }}]\n",
        entry("a", "{ service = \"a.service\" }", "")
    );

    assert_eq!(problem_of(&on_a_service).2, Problem::AlsoWithoutPackage);
    assert_eq!(problem_of(&gamemode("also = []\n")).2, Problem::EmptyAlso);
    assert_eq!(
        problem_of(&gamemode("also = [{ package = \"gamemode\" }]\n")).2,
        Problem::AlsoRepeats("gamemode".to_owned())
    );
    assert_eq!(
        problem_of(&gamemode(
            "also = [{ package = \"lib32-gamemode\" }, { package = \"lib32-gamemode\" }]\n"
        ))
        .2,
        Problem::AlsoRepeats("lib32-gamemode".to_owned())
    );
    assert_eq!(
        problem_of(&gamemode("also = [{ package = \"-rf\" }]\n")).2,
        Problem::BadName(PackageName::new("-rf").unwrap_err())
    );
}

#[test]
fn also_takes_tables_of_a_known_kind_only() {
    for also in [
        "also = [{ group = \"wheel\" }]\n",
        "also = [\"lib32-gamemode\"]\n",
    ] {
        assert!(
            matches!(
                ToggleCatalog::parse(&gamemode(also), "test.toml"),
                Err(CatalogError::Syntax {
                    source_name: _,
                    message: _
                })
            ),
            "{also}"
        );
    }
}

#[test]
fn also_lists_units_of_the_system_and_of_the_user() {
    let text = gamemode(
        "also = [{ package = \"lib32-gamemode\" }, { service = \"podman.socket\" }, { user-service = \"podman.socket\" }]\n",
    );

    let catalog = ToggleCatalog::parse(&text, "test.toml").unwrap();
    let entry = catalog.entry("gamemode").unwrap();
    let socket = UnitName::new("podman.socket").unwrap();

    assert_eq!(entry.also().len(), 1);
    assert_eq!(entry.also_units(), std::slice::from_ref(&socket));
    assert_eq!(entry.also_user_units(), [socket]);
}

#[test]
fn also_units_are_valid_and_named_once_per_scope() {
    assert_eq!(
        problem_of(&gamemode("also = [{ service = \"libvirtd\" }]\n")).2,
        Problem::BadName(UnitName::new("libvirtd").unwrap_err())
    );
    assert_eq!(
        problem_of(&gamemode(
            "also = [{ service = \"a.service\" }, { service = \"a.service\" }]\n"
        ))
        .2,
        Problem::AlsoRepeats("a.service".to_owned())
    );
}

fn office(fresh: &str, still: &str) -> String {
    format!(
        "{SECTION}{}{fresh}{}{still}",
        entry("fresh", "{ package = \"libreoffice-fresh\" }", ""),
        entry("still", "{ package = \"libreoffice-still\" }", "")
    )
}

#[test]
fn conflicts_names_the_entries_turned_off() {
    let text = office("conflicts = [\"still\"]\n", "");
    let catalog = ToggleCatalog::parse(&text, "test.toml").unwrap();

    assert_eq!(catalog.entry("fresh").unwrap().conflicts(), ["still"]);
    assert_eq!(catalog.entry("still").unwrap().conflicts(), [""; 0]);
}

#[test]
fn conflicts_needs_other_entries_of_the_file_each_once() {
    assert_eq!(
        problem_of(&office("conflicts = []\n", "")),
        (
            "test.toml".to_owned(),
            "fresh".to_owned(),
            Problem::EmptyConflicts
        )
    );
    assert_eq!(
        problem_of(&office("conflicts = [\"fresh\"]\n", "")).2,
        Problem::ConflictsItself
    );
    assert_eq!(
        problem_of(&office("", "conflicts = [\"gone\"]\n")).2,
        Problem::UnknownConflict("gone".to_owned())
    );
    assert_eq!(
        problem_of(&office("conflicts = [\"still\", \"still\"]\n", "")).2,
        Problem::ConflictRepeats("still".to_owned())
    );
}

#[test]
fn an_icon_is_a_plain_file_name() {
    let catalog =
        ToggleCatalog::parse(&gamemode("icon = \"Game_mode-2.png\"\n"), "test.toml").unwrap();

    assert_eq!(
        catalog.entry("gamemode").unwrap().icon(),
        Some("Game_mode-2.png")
    );

    for icon in ["", "../steam.png", "icons/steam.png", ".hidden", "a b.png"] {
        assert_eq!(
            problem_of(&gamemode(&format!("icon = \"{icon}\"\n"))).2,
            Problem::BadIcon(icon.to_owned())
        );
    }
}

#[test]
fn the_icon_folder_is_named_after_the_catalog_file() {
    assert_eq!(
        ToggleCatalog::icon_dir(Path::new("/usr/share/x/catalog"), "gaming.toml"),
        Path::new("/usr/share/x/catalog/icons/gaming")
    );
}

#[test]
fn a_url_is_a_plain_https_link() {
    for url in [
        "https://github.com/FeralInteractive/gamemode",
        "https://play0ad.com/",
        "https://example.org:8443/a?b=c#d",
        "https://example.org",
    ] {
        let catalog =
            ToggleCatalog::parse(&gamemode(&format!("url = \"{url}\"\n")), "test.toml").unwrap();

        assert_eq!(catalog.entry("gamemode").unwrap().url(), Some(url));
    }

    assert_eq!(
        ToggleCatalog::parse(&gamemode(""), "test.toml")
            .unwrap()
            .entry("gamemode")
            .unwrap()
            .url(),
        None
    );
}

#[test]
fn a_url_without_https_or_host_or_with_blanks_is_refused() {
    let long = format!("https://example.org/{}", "a".repeat(2048));

    for url in [
        "",
        "http://example.org",
        "file:///etc/passwd",
        "javascript:alert(1)",
        "https://",
        "https:///path",
        "https://user@example.org",
        "https://-example.org",
        "https://example.org:80a",
        "https://example.org/a b",
        "https://example.org/\\u0007",
        "https://example.org https://other.org",
        long.as_str(),
    ] {
        let (source_name, id, problem) = problem_of(&gamemode(&format!("url = \"{url}\"\n")));
        let shown = url.replace("\\u0007", "\u{7}");

        assert_eq!(source_name, "test.toml");
        assert_eq!(id, "gamemode");
        assert_eq!(problem, Problem::BadUrl(shown), "{url}");
    }
}

#[test]
fn conflicts_names_no_entry_the_entry_needs() {
    let text = office(
        "requires = { all = [\"still\"] }\nconflicts = [\"still\"]\n",
        "",
    );

    assert_eq!(
        problem_of(&text),
        (
            "test.toml".to_owned(),
            "fresh".to_owned(),
            Problem::ConflictsRequirement("still".to_owned())
        )
    );
}
