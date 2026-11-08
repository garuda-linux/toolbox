use std::io::Write as _;

use flate2::Compression;
use flate2::write::GzEncoder;

use super::{DumpError, parse};

const LIMIT: u64 = 1 << 20;

const FULL: &str = r#"[{
  "ID": 1, "Name": "paru", "PackageBaseID": 2, "PackageBase": "paru",
  "Version": "2.0.4-1", "Description": "Feature packed AUR helper",
  "URL": "https://github.com/morganamilo/paru", "NumVotes": 1234,
  "Popularity": 45.67, "OutOfDate": null, "Maintainer": "Morganamilo",
  "Submitter": "Morganamilo", "FirstSubmitted": 1600000000,
  "LastModified": 1700000000, "URLPath": "/cgit/aur.git/snapshot/paru.tar.gz",
  "Depends": ["git", "pacman>=6.1"], "MakeDepends": ["cargo"],
  "OptDepends": ["bat: colored pkgbuild printing"], "CheckDepends": ["x"],
  "Provides": ["paru"], "Conflicts": ["paru-bin"], "Replaces": ["y"],
  "Groups": ["z"], "License": ["GPL-3.0-or-later"], "Keywords": ["helper", "aur"],
  "CoMaintainers": ["someone"]
}]"#;

pub fn gzipped(json: &str) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(json.as_bytes()).unwrap();
    encoder.finish().unwrap()
}

#[test]
fn every_kept_field_is_read() {
    let packages = parse(gzipped(FULL).as_slice(), LIMIT).unwrap();

    let [paru] = packages.as_slice() else {
        panic!("one package expected, got {packages:?}");
    };

    assert_eq!(paru.name.as_str(), "paru");
    assert_eq!(paru.version, "2.0.4-1");
    assert_eq!(paru.description, "Feature packed AUR helper");
    assert_eq!(
        paru.url.as_deref(),
        Some("https://github.com/morganamilo/paru")
    );
    assert_eq!(paru.votes, 1234);
    assert!((paru.popularity - 45.67).abs() < 1e-9_f64);
    assert_eq!(paru.out_of_date, None);
    assert_eq!(paru.maintainer.as_deref(), Some("Morganamilo"));
    assert_eq!(paru.first_submitted, 1_600_000_000);
    assert_eq!(paru.last_modified, 1_700_000_000);
    assert_eq!(paru.depends, ["git", "pacman>=6.1"]);
    assert_eq!(paru.make_depends, ["cargo"]);
    assert_eq!(paru.opt_depends, ["bat: colored pkgbuild printing"]);
    assert_eq!(paru.provides, ["paru"]);
    assert_eq!(paru.conflicts, ["paru-bin"]);
    assert_eq!(paru.licenses, ["GPL-3.0-or-later"]);
    assert_eq!(paru.keywords, ["helper", "aur"]);
}

#[test]
fn nulls_and_missing_fields_read_as_empty() {
    let json = r#"[{"Name": "bare", "Version": "1-1", "Description": null,
        "URL": null, "Maintainer": null, "OutOfDate": 1710000000,
        "NumVotes": 0, "Popularity": 0, "FirstSubmitted": 1, "LastModified": 2}]"#;

    let packages = parse(gzipped(json).as_slice(), LIMIT).unwrap();

    let [bare] = packages.as_slice() else {
        panic!("one package expected, got {packages:?}");
    };

    assert_eq!(bare.description, "");
    assert_eq!(bare.url, None);
    assert_eq!(bare.maintainer, None);
    assert_eq!(bare.out_of_date, Some(1_710_000_000));
    assert!(bare.depends.is_empty() && bare.keywords.is_empty());
}

#[test]
fn texts_are_cleaned_and_cut() {
    let long = "d".repeat(2000);
    let json = format!(
        r#"[{{"Name": "evil", "Version": "1\u001b[2J-1",
        "Description": "{long}", "Maintainer": "mal\u202elory",
        "Depends": ["glibc\u0007"], "FirstSubmitted": 1, "LastModified": 2}}]"#
    );

    let packages = parse(gzipped(&json).as_slice(), LIMIT).unwrap();

    let [evil] = packages.as_slice() else {
        panic!("one package expected, got {packages:?}");
    };

    assert_eq!(evil.version, "1[2J-1");
    assert_eq!(evil.description.chars().count(), 512);
    assert_eq!(evil.maintainer.as_deref(), Some("mallory"));
    assert_eq!(evil.depends, ["glibc"]);
}

#[test]
fn an_entry_with_an_invalid_name_is_skipped() {
    let json = r#"[{"Name": "--overwrite", "Version": "1"},
        {"Name": "vim;rm", "Version": "1"}, {"Name": null},
        {"Name": "good", "Version": "1"}]"#;

    let packages = parse(gzipped(json).as_slice(), LIMIT).unwrap();

    let names: Vec<&str> = packages
        .iter()
        .map(|package| package.name.as_str())
        .collect();

    assert_eq!(names, ["good"]);
}

#[test]
fn packages_are_sorted_by_name() {
    let json = r#"[{"Name": "zsh-x"}, {"Name": "alpha"}, {"Name": "mid"}]"#;

    let packages = parse(gzipped(json).as_slice(), LIMIT).unwrap();

    let names: Vec<&str> = packages
        .iter()
        .map(|package| package.name.as_str())
        .collect();

    assert_eq!(names, ["alpha", "mid", "zsh-x"]);
}

#[test]
fn votes_are_clamped_and_a_bad_popularity_is_zero() {
    let json = r#"[{"Name": "a", "NumVotes": 99999999999, "Popularity": -3}]"#;

    let packages = parse(gzipped(json).as_slice(), LIMIT).unwrap();

    assert_eq!(packages[0].votes, u32::MAX);
    assert!(packages[0].popularity.abs() < f64::EPSILON);
}

#[test]
fn a_dump_larger_than_the_limit_is_refused() {
    let json = format!(
        r#"[{{"Name": "a", "Description": "{}"}}]"#,
        "x".repeat(4096)
    );

    assert_eq!(
        parse(gzipped(&json).as_slice(), 1024),
        Err(DumpError::TooLarge(1024))
    );
}

#[test]
fn something_else_than_gzipped_json_is_malformed() {
    assert!(matches!(
        parse(b"not gzip".as_slice(), LIMIT),
        Err(DumpError::Malformed(_))
    ));
    assert!(matches!(
        parse(gzipped(r#"{"Name": "a"}"#).as_slice(), LIMIT),
        Err(DumpError::Malformed(_))
    ));
}
