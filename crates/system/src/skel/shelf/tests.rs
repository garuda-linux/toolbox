use std::fs;
use std::path::Path;

use toolbox_catalog::SkelCatalog;

use super::{Outcome, Shelf, SkelRow, offered, reset};

const CATALOG: &str = r#"
[[config]]
id = "bash"
name = "Bash"
files = ["/etc/skel/.bashrc_garuda", "/etc/skel/.bash_profile"]

[[config]]
id = "fish"
name = "Fish"
files = ["/etc/skel/.config/fish/config.fish"]

[[config]]
id = "zsh"
name = "ZSH"
files = ["/etc/skel/.zshrc"]
"#;

fn catalog() -> SkelCatalog {
    SkelCatalog::parse(CATALOG, "skel.toml").unwrap()
}

/// A skel folder with the files of Bash and Fish, none of ZSH.
fn skel_folder() -> tempfile::TempDir {
    let folder = tempfile::tempdir().unwrap();

    fs::create_dir_all(folder.path().join(".config/fish")).unwrap();
    fs::write(folder.path().join(".config/fish/config.fish"), "fish").unwrap();
    fs::write(folder.path().join(".bashrc_garuda"), "bash").unwrap();
    fs::write(folder.path().join(".bash_profile"), "profile").unwrap();
    folder
}

fn row(entry: &str, name: &str, files: &[&str]) -> SkelRow {
    SkelRow {
        entry: entry.to_owned(),
        name: name.to_owned(),
        files: files.iter().map(|&file| file.to_owned()).collect(),
        targets: files
            .iter()
            .map(|file| file.replacen("/etc/skel/", "~/", 1))
            .collect(),
    }
}

/// A whole reset, in the two steps the app runs around its thread.
fn reset_chosen(shelf: &mut Shelf, catalog: &SkelCatalog, skel: &Path, home: &Path) {
    if let Some(entries) = shelf.begin_reset(catalog) {
        shelf.finish_reset(reset(&entries, skel, home));
    }
}

fn fish_chosen(skel: &tempfile::TempDir) -> (SkelCatalog, Shelf) {
    let catalog = catalog();
    let mut shelf = Shelf::default();

    shelf.stock(offered(&catalog, skel.path()));
    shelf.choose("fish", true);
    (catalog, shelf)
}

fn shelf() -> Shelf {
    let mut shelf = Shelf::default();

    shelf.stock(vec![
        row(
            "bash",
            "Bash",
            &["/etc/skel/.bashrc_garuda", "/etc/skel/.bash_profile"],
        ),
        row("fish", "Fish", &["/etc/skel/.config/fish/config.fish"]),
    ]);
    shelf
}

#[test]
fn only_entries_with_a_file_in_skel_are_offered() {
    let skel = skel_folder();
    let rows = offered(&catalog(), skel.path());
    let path = |file: &str| skel.path().join(file).display().to_string();

    assert_eq!(
        rows,
        [
            SkelRow {
                entry: "bash".to_owned(),
                name: "Bash".to_owned(),
                files: vec![path(".bashrc_garuda"), path(".bash_profile")],
                targets: vec!["~/.bashrc_garuda".to_owned(), "~/.bash_profile".to_owned()],
            },
            SkelRow {
                entry: "fish".to_owned(),
                name: "Fish".to_owned(),
                files: vec![path(".config/fish/config.fish")],
                targets: vec!["~/.config/fish/config.fish".to_owned()],
            },
        ]
    );
}

#[test]
fn nothing_is_chosen_at_first() {
    let shelf = shelf();

    assert_eq!(shelf.chosen_count(), 0);
    assert_eq!(shelf.chosen_targets(), Vec::<String>::new());
    assert!(!shelf.is_chosen("bash"));
    assert_eq!(*shelf.outcome(), Outcome::default());
}

#[test]
fn choosing_counts_each_entry_once() {
    let mut shelf = shelf();

    shelf.choose("fish", true);
    shelf.choose("fish", true);

    assert_eq!(shelf.chosen_count(), 1);
    assert!(shelf.is_chosen("fish"));

    shelf.choose("fish", false);

    assert_eq!(shelf.chosen_count(), 0);
    assert!(!shelf.is_chosen("fish"));
}

#[test]
fn an_entry_the_page_does_not_offer_cannot_be_chosen() {
    let mut shelf = shelf();

    shelf.choose("zsh", true);

    assert_eq!(shelf.chosen_count(), 0);
}

#[test]
fn an_entry_that_left_the_list_is_no_longer_chosen() {
    let mut shelf = shelf();

    shelf.choose("fish", true);
    shelf.choose("bash", true);
    shelf.stock(vec![row("bash", "Bash", &["/etc/skel/.bashrc_garuda"])]);

    assert_eq!(shelf.chosen_count(), 1);
    assert!(shelf.is_chosen("bash"));
    assert!(!shelf.is_chosen("fish"));
}

#[test]
fn a_reset_copies_the_chosen_entries_and_forgets_the_choice() {
    let skel = skel_folder();
    let home = tempfile::tempdir().unwrap();
    let (catalog, mut shelf) = fish_chosen(&skel);

    reset_chosen(&mut shelf, &catalog, skel.path(), home.path());

    assert_eq!(
        *shelf.outcome(),
        Outcome {
            done: true,
            copied: 1,
            failed: Vec::new(),
        }
    );
    assert_eq!(shelf.chosen_count(), 0);
    assert_eq!(
        fs::read_to_string(home.path().join(".config/fish/config.fish")).unwrap(),
        "fish"
    );
    assert!(
        !home.path().join(".bashrc_garuda").exists(),
        "an entry that was not chosen was reset"
    );
}

#[test]
fn a_reset_that_fails_names_the_entry() {
    let skel = skel_folder();
    let (catalog, mut shelf) = fish_chosen(&skel);

    reset_chosen(
        &mut shelf,
        &catalog,
        skel.path(),
        Path::new("/nonexistent/home"),
    );

    assert_eq!(
        *shelf.outcome(),
        Outcome {
            done: true,
            copied: 0,
            failed: vec!["Fish".to_owned()],
        }
    );
}

#[test]
fn a_reset_of_nothing_reports_nothing() {
    let skel = skel_folder();
    let home = tempfile::tempdir().unwrap();
    let mut shelf = shelf();

    reset_chosen(&mut shelf, &catalog(), skel.path(), home.path());

    assert_eq!(*shelf.outcome(), Outcome::default());
}

#[test]
fn a_reset_is_busy_from_its_start_until_its_outcome_arrives() {
    let skel = skel_folder();
    let catalog = catalog();
    let mut shelf = Shelf::default();

    shelf.stock(offered(&catalog, skel.path()));
    assert_eq!(shelf.begin_reset(&catalog), None, "nothing is chosen");
    assert!(!shelf.resetting());

    shelf.choose("fish", true);

    let entries = shelf.begin_reset(&catalog).unwrap();

    assert_eq!(entries.len(), 1);
    assert!(shelf.resetting());
    assert!(
        shelf.is_chosen("fish"),
        "the choice shows while it is reset"
    );
    assert_eq!(shelf.begin_reset(&catalog), None, "a second reset started");

    shelf.finish_reset(Outcome::stopped(&entries));

    assert!(!shelf.resetting());
    assert_eq!(shelf.chosen_count(), 0);
    assert_eq!(
        *shelf.outcome(),
        Outcome {
            done: true,
            copied: 0,
            failed: vec!["Fish".to_owned()],
        }
    );
}

#[test]
fn the_question_names_the_home_files_a_reset_replaces() {
    let mut shelf = shelf();
    shelf.choose("fish", true);
    shelf.choose("bash", true);

    assert_eq!(
        shelf.chosen_targets(),
        [
            "~/.bashrc_garuda",
            "~/.bash_profile",
            "~/.config/fish/config.fish"
        ]
    );
}
