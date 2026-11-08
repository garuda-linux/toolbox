//! Rows of symlinked files: a link is edited like any root-owned file and
//! names its target, and a link the helper would refuse says why instead.

use std::collections::BTreeSet;

use toolbox_plan::{ConfigFilePath, FetchRefusal};

use super::super::{Found, Link, Listing, Row, Target, rows, target};
use super::catalog;

fn row(found: Found, id: &str) -> Row {
    let mut listing = Listing::default();

    listing.insert(id, found);
    rows(&catalog(), &listing, &BTreeSet::new(), "")
        .into_iter()
        .find(|row| row.key == id)
        .unwrap()
}

#[test]
fn a_link_to_an_editable_file_is_edited_and_names_its_target() {
    let linked = row(
        Found::File {
            link: Some(Link::to(
                &ConfigFilePath::new("/etc/default/grub.real").unwrap(),
            )),
        },
        "grub",
    );

    assert_eq!(
        (linked.action, linked.admin, linked.link, linked.refusal),
        ("edit", true, true, "")
    );
    assert_eq!(linked.path, "/etc/default/grub");
    assert_eq!(linked.target, "/etc/default/grub.real");
}

#[test]
fn a_refused_link_offers_nothing_and_carries_the_reason() {
    for (reason, code) in [
        (FetchRefusal::Outside, "outside"),
        (FetchRefusal::Dangling, "dangling"),
        (FetchRefusal::NotRegular, "not-regular"),
    ] {
        let linked = row(
            Found::File {
                link: Some(Link::refused("/usr/lib/os-release", reason)),
            },
            "os-release",
        );

        assert_eq!(
            (linked.action, linked.admin, linked.link, linked.refusal),
            ("", true, true, code)
        );
        assert_eq!(linked.target, "/usr/lib/os-release");
    }
}

#[test]
fn a_plain_file_has_no_target() {
    let plain = row(Found::File { link: None }, "grub");

    assert_eq!(
        (
            plain.action,
            plain.link,
            plain.target.as_str(),
            plain.refusal
        ),
        ("edit", false, "", "")
    );
}

/// The helper decides again when the edit starts, so the key of a link
/// still stands for the path of the catalog.
#[test]
fn the_key_of_a_link_is_the_path_of_the_catalog() {
    let mut listing = Listing::default();

    listing.insert(
        "os-release",
        Found::File {
            link: Some(Link::refused("/usr/lib/os-release", FetchRefusal::Outside)),
        },
    );

    assert_eq!(
        target(&catalog(), &listing, "os-release"),
        Some(Target::Root(
            ConfigFilePath::new("/etc/os-release").unwrap()
        ))
    );
}
