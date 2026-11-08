use super::super::ToggleCatalog;
use super::{SECTION, entry, package};

#[test]
fn config_edits_lists_the_user_config_checks() {
    let text = format!(
        "{SECTION}{}{}",
        package("docker", ""),
        entry(
            "discord",
            "{ user-json = { path = \".config/discord/settings.json\", key = \"SKIP_HOST_UPDATE\" } }",
            ""
        ),
    );

    let catalog = ToggleCatalog::parse(&text, "test.toml").unwrap();

    assert_eq!(catalog.config_edits().count(), 1);
    assert!(catalog.entry("docker").is_some());
}
