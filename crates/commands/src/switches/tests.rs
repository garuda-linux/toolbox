use toolbox_catalog::ToggleCatalog;
use toolbox_cli::output::{Paint, Style};

use super::{shown, text};
use crate::lists::ToggleRow;

const CATALOG: &str = r#"
[[section]]
id = "containers"
title = "Containers"
icon = "x"

[[section.entry]]
id = "docker"
title = "Docker"
description = "The engine."
check = { package = "docker" }
also = [{ package = "docker-compose" }, { service = "docker.socket" }]

[[section.entry]]
id = "podman"
title = "Podman"
description = "The other engine."
check = { package = "podman" }
"#;

fn row(id: &str, state: &'static str) -> ToggleRow {
    ToggleRow {
        page: "components",
        section: "containers".to_owned(),
        id: id.to_owned(),
        name: id.to_owned(),
        state,
        available: true,
    }
}

#[test]
fn a_switch_shows_its_page_its_state_and_what_it_brings() {
    let catalog = ToggleCatalog::parse(CATALOG, "components.toml").unwrap();
    let docker = row("docker", "on");
    let shown = shown(&docker, catalog.entry("docker").unwrap());

    assert_eq!(
        text(Paint::new(Style::Plain), &shown),
        "docker\n\n  \
         description  The engine.\n  \
         id           docker\n  \
         page         components\n  \
         section      containers\n  \
         state        on\n  \
         available    yes\n  \
         checks       package docker\n  \
         brings       docker-compose, docker.socket\n"
    );
}

#[test]
fn a_switch_that_brings_nothing_says_none() {
    let catalog = ToggleCatalog::parse(CATALOG, "components.toml").unwrap();
    let podman = row("podman", "unknown");
    let text = text(
        Paint::new(Style::Plain),
        &shown(&podman, catalog.entry("podman").unwrap()),
    );

    assert!(text.contains("brings       none\n"), "{text}");
    assert!(text.contains("state        unknown\n"), "{text}");
}

#[test]
fn the_json_is_the_row_of_the_list_and_what_the_catalog_says() {
    let catalog = ToggleCatalog::parse(CATALOG, "components.toml").unwrap();
    let docker = row("docker", "off");
    let shown = shown(&docker, catalog.entry("docker").unwrap());
    let value = serde_json::to_value(&shown).unwrap();

    assert_eq!(value["id"], "docker");
    assert_eq!(value["on"], false);
    assert_eq!(value["check"], "package docker");
    assert_eq!(value["also"][1], "docker.socket");
}
