use std::io::{Read as _, Write as _};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

use super::{AurPackage, DumpError, parse};

const LIMIT: u64 = 1 << 20;

/// The first five entries of `packages-meta-ext-v1.json.gz` as the AUR
/// served it, sorted by name.
pub const DUMP: &[u8] = include_bytes!("fixtures/packages-meta-ext-v1.json.gz");

pub fn gzipped(json: &str) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(json.as_bytes()).unwrap();
    encoder.finish().unwrap()
}

pub fn unpacked() -> String {
    let mut json = String::new();

    GzDecoder::new(DUMP).read_to_string(&mut json).unwrap();
    json
}

fn package<'packages>(packages: &'packages [AurPackage], name: &str) -> &'packages AurPackage {
    packages
        .iter()
        .find(|package| package.name.as_str() == name)
        .unwrap_or_else(|| panic!("{name} is in the dump"))
}

#[test]
fn every_kept_field_is_read() {
    let packages = parse(DUMP, LIMIT).unwrap();
    let ime = package(&packages, "0c-ime");

    assert_eq!(ime.version, "0.2.9-1");
    assert_eq!(
        ime.description,
        "Zhuyin (Bopomofo) input method with statistical and neural candidate selection, \
         for Fcitx5 and IBus"
    );
    assert_eq!(
        ime.url.as_deref(),
        Some("https://github.com/0c-biggo/0c-ime")
    );
    assert_eq!(ime.votes, 0);
    assert_eq!(ime.out_of_date, Some(1_794_068_437));
    assert_eq!(ime.maintainer.as_deref(), Some("neo_chen"));
    assert_eq!(ime.first_submitted, 1_792_590_299);
    assert_eq!(ime.last_modified, 1_793_978_572);
    assert_eq!(ime.depends.len(), 11);
    assert_eq!(ime.depends.first().map(String::as_str), Some("cairo"));
    assert_eq!(
        ime.make_depends,
        ["go", "rust", "pkgconf", "fcitx5", "ibus"]
    );
    assert_eq!(
        ime.opt_depends,
        [
            "fcitx5",
            "ibus",
            "python",
            "python-numpy",
            "python-onnxruntime"
        ]
    );
    assert_eq!(ime.licenses.len(), 7);

    let tools = package(&packages, "0xtools");

    assert_eq!(tools.votes, 1);
    assert!((tools.popularity - 0.000_604).abs() < 1e-9_f64);

    assert_eq!(
        package(&packages, "10mb.video-git").provides,
        ["10mb.video"]
    );
    assert_eq!(
        package(&packages, "10ff-git").keywords,
        ["typing", "typing-test"]
    );
}

#[test]
fn nulls_and_missing_fields_read_as_empty() {
    // `0ad-boongui` has no maintainer, no out-of-date mark and no
    // dependencies. Its description is edited to null.
    let json = unpacked().replacen(
        r#""Description":"User interface mod for the RTS game 0 A.D.""#,
        r#""Description":null"#,
        1,
    );

    let packages = parse(gzipped(&json).as_slice(), LIMIT).unwrap();
    let bare = package(&packages, "0ad-boongui");

    assert_eq!(bare.description, "");
    assert_eq!(bare.maintainer, None);
    assert_eq!(bare.out_of_date, None);
    assert!(bare.depends.is_empty() && bare.keywords.is_empty());
    assert!(bare.provides.is_empty() && bare.conflicts.is_empty());
}

#[test]
fn packages_are_sorted_by_name() {
    // The dump's entries edited into the reverse order.
    let mut entries: Vec<serde_json::Value> = serde_json::from_str(&unpacked()).unwrap();

    entries.reverse();

    let reversed = serde_json::to_string(&entries).unwrap();
    let packages = parse(gzipped(&reversed).as_slice(), LIMIT).unwrap();
    let names: Vec<&str> = packages
        .iter()
        .map(|package| package.name.as_str())
        .collect();

    assert_eq!(
        names,
        [
            "0ad-boongui",
            "0c-ime",
            "0xtools",
            "10ff-git",
            "10mb.video-git"
        ]
    );
}

#[test]
fn a_dump_larger_than_the_limit_is_refused() {
    assert!(unpacked().len() > 1024);
    assert_eq!(parse(DUMP, 1024), Err(DumpError::TooLarge(1024)));
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
fn votes_are_clamped_and_a_bad_popularity_is_zero() {
    let json = r#"[{"Name": "a", "NumVotes": 99999999999, "Popularity": -3}]"#;

    let packages = parse(gzipped(json).as_slice(), LIMIT).unwrap();

    assert_eq!(packages[0].votes, u32::MAX);
    assert!(packages[0].popularity.abs() < f64::EPSILON);
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
