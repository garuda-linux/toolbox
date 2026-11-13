use std::fs;
use std::path::Path;

use tempfile::TempDir;
use toolbox_catalog::SkelCatalog;
use toolbox_cli::Exit;
use toolbox_cli::args::Confirm;
use toolbox_cli::confirm::Prompt;

use super::{Folders, carry_out, chosen, ids};
use crate::change::Refused;
use crate::out::{Out, Settings};

const FISH: &str = ".config/fish/config.fish";

/// Answers every question with `answer`.
struct Answer(&'static str);

impl Prompt for Answer {
    fn is_terminal(&self) -> bool {
        true
    }

    fn ask(&mut self, _question: &str) -> Option<String> {
        Some(self.0.to_owned())
    }
}

fn catalog() -> SkelCatalog {
    SkelCatalog::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog")).unwrap()
}

/// A skel folder with Garuda's fish settings, a home with the user's own.
fn folders() -> (TempDir, TempDir) {
    let skel = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();

    for (root, text) in [(&skel, "garuda\n"), (&home, "mine\n")] {
        let file = root.path().join(FISH);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, text).unwrap();
    }

    (skel, home)
}

fn reset_with(confirm: Confirm, answer: &'static str, skel: &TempDir, home: &TempDir) -> Exit {
    reset_out(confirm, answer, skel, home).0
}

fn reset_out(
    confirm: Confirm,
    answer: &'static str,
    skel: &TempDir,
    home: &TempDir,
) -> (Exit, String) {
    let entries = chosen(&["fish".to_owned()], &catalog(), skel.path()).unwrap();
    let folders = Folders {
        skel_root: skel.path(),
        home: home.path(),
    };

    let mut sink = Vec::new();

    let exit = carry_out(
        &entries,
        confirm,
        &folders,
        &mut Answer(answer),
        &mut Out::new(&mut sink, Settings::default()),
    );

    (exit, String::from_utf8(sink).unwrap())
}

#[test]
fn a_blank_line_sets_the_question_apart_from_the_files() {
    let (skel, home) = folders();

    let (_, out) = reset_out(Confirm::default(), "n", &skel, &home);

    assert!(out.ends_with("kept as a backup beside it.\n\n"), "{out:?}");
}

/// Completion offers only what the skel folder has.
#[test]
fn the_ids_are_the_settings_skel_has() {
    let (skel, _home) = folders();

    assert_eq!(ids(&catalog(), skel.path()), ["fish"]);
}

fn home_fish(home: &TempDir) -> String {
    fs::read_to_string(home.path().join(FISH)).unwrap()
}

fn files_beside(home: &TempDir) -> usize {
    fs::read_dir(home.path().join(".config/fish"))
        .unwrap()
        .count()
}

#[test]
fn a_reset_copies_garudas_settings_and_keeps_the_old_file() {
    let (skel, home) = folders();

    assert_eq!(
        reset_with(Confirm::default(), "y", &skel, &home),
        Exit::Success
    );
    assert_eq!(home_fish(&home), "garuda\n");
    assert_eq!(files_beside(&home), 2, "the backup beside it");
}

#[test]
fn a_declined_or_dry_reset_changes_nothing() {
    let (skel, home) = folders();
    let dry = Confirm {
        dry_run: true,
        ..Confirm::default()
    };

    assert_eq!(
        reset_with(Confirm::default(), "n", &skel, &home),
        Exit::Declined
    );
    assert_eq!(reset_with(dry, "y", &skel, &home), Exit::Success);
    assert_eq!(home_fish(&home), "mine\n");
}

/// Only what /etc/skel holds is offered, as on the page.
#[test]
fn settings_skel_lacks_or_no_entry_names_are_refused() {
    let (skel, _home) = folders();

    for id in ["bash", "fsh"] {
        let refused = chosen(&[id.to_owned()], &catalog(), skel.path()).unwrap_err();
        assert!(matches!(refused, Refused::Unknown(_)), "{id}: {refused:?}");
    }

    let hinted = chosen(&["fsh".to_owned()], &catalog(), skel.path()).unwrap_err();
    assert!(hinted.message().contains("fish"), "{hinted:?}");
}
