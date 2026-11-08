use std::path::Path;

use toolbox_cli::complete::Names;

use super::{read, write};

fn names_at(dir: &Path) -> Vec<String> {
    read(dir, Names::Apps, 100).unwrap_or_default()
}

#[test]
fn a_written_list_comes_back() {
    let dir = tempfile::tempdir().unwrap();

    write(
        dir.path(),
        Names::Apps,
        100,
        &["firefox".to_owned(), "gimp".to_owned()],
    );

    assert_eq!(names_at(dir.path()), ["firefox", "gimp"]);
}

#[test]
fn a_list_older_than_the_ttl_is_ignored() {
    let dir = tempfile::tempdir().unwrap();

    write(dir.path(), Names::Apps, 10, &["firefox".to_owned()]);

    assert_eq!(
        read(dir.path(), Names::Apps, 10 + super::TTL),
        Some(vec!["firefox".to_owned()])
    );
    assert_eq!(read(dir.path(), Names::Apps, 10 + super::TTL + 1), None);
}

#[test]
fn a_file_of_an_older_version_is_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(super::file(Names::Apps));
    std::fs::create_dir_all(dir.path()).unwrap();
    std::fs::write(&path, "0 100\nfirefox\n").unwrap();

    assert_eq!(names_at(dir.path()), Vec::<String>::new());
    assert_eq!(read(dir.path(), Names::Apps, 100), None);
}

#[test]
fn each_kind_of_name_has_its_own_file() {
    assert_ne!(super::file(Names::Apps), super::file(Names::Shells));

    let stem = super::file(Names::Services { user: true });
    let stem = stem.strip_suffix(".names").unwrap();
    assert!(
        stem.chars().all(|character| character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || character == '-'),
        "{stem}"
    );
}

#[test]
fn a_stamped_list_comes_back_for_its_stamp_only_whatever_its_age() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), Names::AurPackages, 100, &["paru".to_owned()]);

    assert_eq!(
        super::read_stamped(dir.path(), Names::AurPackages, 100),
        Some(vec!["paru".to_owned()])
    );
    assert_eq!(
        super::read_stamped(dir.path(), Names::AurPackages, 101),
        None
    );
}
