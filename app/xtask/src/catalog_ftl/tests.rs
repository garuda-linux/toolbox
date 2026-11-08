use std::collections::HashMap;

use toolbox_catalog::{ConfigFileCatalog, ToggleCatalog};

use super::{config_messages, kept, messages, render};
use crate::translations::plain_value;

const CATALOG: &str = r#"
[[section]]
id = "audio"
title = "Audio"
icon = "audio-volume-high"

[[section.entry]]
id = "pipewire-support"
title = "PipeWire"
description = "Low latency {audio}."
check = { package = "pipewire-support" }
"#;

fn catalog() -> ToggleCatalog {
    ToggleCatalog::parse(CATALOG, "components.toml").unwrap()
}

#[test]
fn sections_and_entries_get_ids_from_the_file_and_their_own() {
    let ids: Vec<(String, String)> = messages("components", &catalog())
        .into_iter()
        .map(|message| (message.id, message.english))
        .collect();

    assert_eq!(
        ids,
        vec![
            ("catalog-components-audio".to_owned(), "Audio".to_owned()),
            (
                "catalog-components-pipewire-support".to_owned(),
                "PipeWire".to_owned()
            ),
            (
                "catalog-components-pipewire-support-description".to_owned(),
                "Low latency {audio}.".to_owned()
            ),
        ]
    );
}

#[test]
fn braces_and_leading_syntax_are_escaped() {
    assert_eq!(plain_value("Low {x} y"), r#"Low {"{"}x{"}"} y"#);
    assert_eq!(plain_value("[beta] tools"), r#"{"["}beta] tools"#);
    assert_eq!(plain_value("*nix"), r#"{"*"}nix"#);
    assert_eq!(plain_value(".NET"), r#"{"."}NET"#);
    assert_eq!(plain_value("plain"), "plain");
}

#[test]
fn a_missing_translation_keeps_the_english_marked() {
    let all = messages("components", &catalog());
    let text = render("de-DE", &all, |message| {
        (message.id == "catalog-components-audio").then(|| "Klang".to_owned())
    });

    assert!(text.contains("\ncatalog-components-audio = Klang\n"));
    assert!(text.contains("\n# untranslated\ncatalog-components-pipewire-support = PipeWire\n"));
}

#[test]
fn english_is_never_marked() {
    let all = messages("components", &catalog());
    let text = render("en-US", &all, |_| None);

    assert!(!text.contains("untranslated"));
    assert!(text.contains("\ncatalog-components-audio = Audio\n"));
}

#[test]
fn kept_translations_skip_the_untranslated_ones() {
    let file = "# head\n\ncatalog-a = Eins\n# untranslated\ncatalog-b = Two\n";

    assert_eq!(
        kept(file),
        HashMap::from([("catalog-a".to_owned(), "Eins".to_owned())])
    );
}

#[test]
fn config_files_give_their_sections_names_and_descriptions() {
    let catalog = ConfigFileCatalog::parse(
        r#"
[[section]]
id = "audio"
title = "Audio"
icon = "audio-volume-high"

[[section.entry]]
id = "alsa-base"
name = "ALSA Base"
path = "/etc/modprobe.d/alsa-base.conf"
description = "ALSA kernel module options"

[[section.entry]]
id = "asoundrc"
name = ".asoundrc"
path = "~/.asoundrc"
"#,
        "config-files.toml",
    )
    .unwrap();

    let ids: Vec<(String, String)> = config_messages("config-files", &catalog)
        .into_iter()
        .map(|message| (message.id, message.english))
        .collect();

    assert_eq!(
        ids,
        [
            ("catalog-config-files-audio", "Audio"),
            ("catalog-config-files-alsa-base", "ALSA Base"),
            (
                "catalog-config-files-alsa-base-description",
                "ALSA kernel module options"
            ),
            ("catalog-config-files-asoundrc", ".asoundrc"),
        ]
        .map(|(id, english)| (id.to_owned(), english.to_owned()))
    );
}

#[test]
fn colour_schemes_give_their_names() {
    let source = crate::root().join(crate::catalogs::SOURCE_DIR);
    let catalog = toolbox_catalog::ColourSchemeCatalog::load(&source).unwrap();

    let messages = super::scheme_messages("colour-schemes", &catalog);

    assert_eq!(messages.len(), catalog.schemes().len());
    assert!(messages.contains(&super::Message {
        id: "catalog-colour-schemes-nord-light".to_owned(),
        english: "Nord light".to_owned(),
    }));
}
