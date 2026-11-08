use std::collections::BTreeSet;
use std::fs;

use super::{catalog_names, check, theme_names};

#[test]
fn a_catalog_icon_without_a_file_ending_is_a_theme_name() {
    assert_eq!(
        catalog_names("icon = \"audio-volume-high\"\nicon = \"bottles.svg\"\nname = \"x\"\n"),
        ["audio-volume-high"]
    );
}

#[test]
fn the_theme_lists_every_icon_file_and_link_by_name() {
    let dir = tempfile::tempdir().expect("temporary folder");
    let actions = dir.path().join("actions/22");

    fs::create_dir_all(&actions).expect("icon folder");
    fs::write(actions.join("go-home.svg"), "").expect("icon");
    fs::write(actions.join("index.theme"), "").expect("other file");
    std::os::unix::fs::symlink("go-home.svg", actions.join("user-home.svg")).expect("link");

    assert_eq!(
        theme_names(dir.path()),
        BTreeSet::from(["go-home".to_owned(), "user-home".to_owned()])
    );
}

#[test]
fn a_name_the_theme_lacks_is_reported_with_its_place() {
    let theme = BTreeSet::from(["go-home".to_owned()]);
    let used = vec![
        ("components.toml".to_owned(), "go-home".to_owned()),
        (
            "gaming.toml".to_owned(),
            "view-app-grid-symbolic".to_owned(),
        ),
    ];

    assert_eq!(
        check(&used, &theme),
        ["gaming.toml: `view-app-grid-symbolic` is not in the Breeze icon theme"]
    );
}
