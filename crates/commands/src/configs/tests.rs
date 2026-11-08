use toolbox_cli::Exit;
use toolbox_cli::output::Mark;
use toolbox_core::features::configs::session::EditedFile;
use toolbox_plan::{ConfigFilePath, ConfigText, HomePath, PlannedAction, RootAction, UserAction};
use toolbox_system::ReadProblem;

use super::{FileRow, file_listing, read_problem, unknown, write_back};

#[test]
fn a_key_the_catalog_lacks_ends_as_not_available() {
    let mut out = Vec::new();
    assert_eq!(unknown("nope", &mut out), Exit::Unavailable);
}

fn text(content: &str) -> ConfigText {
    ConfigText::new(content.to_owned()).unwrap()
}

fn root(path: &str) -> EditedFile {
    ConfigFilePath::new(path).unwrap().into()
}

#[test]
fn an_unchanged_copy_writes_nothing() {
    assert!(write_back(&root("/etc/pacman.conf"), &text("a\n"), text("a\n")).is_none());
}

#[test]
fn a_changed_copy_of_a_file_of_the_user_is_written_as_the_user() {
    let path = HomePath::new(".bashrc").unwrap();
    let original = text("a\n");
    let plan = write_back(&path.clone().into(), &original, text("b\n")).unwrap();

    let PlannedAction::User(UserAction::WriteHomeFile { write }) = &plan.steps()[0].action else {
        panic!("one write of the user");
    };

    assert_eq!(write.path, path);
    assert_eq!(write.original, original.fingerprint());
}

#[test]
fn a_changed_copy_is_written_against_the_original_s_fingerprint() {
    let original = text("a\n");
    let plan = write_back(&root("/etc/pacman.conf"), &original, text("b\n")).unwrap();

    let PlannedAction::Root(RootAction::WriteConfigFile { write }) = &plan.steps()[0].action else {
        panic!("one root write");
    };

    assert_eq!(write.original, original.fingerprint());

    assert_eq!(write.content.as_str(), "b\n");
}

fn file(key: &str, admin: bool, read_only: bool) -> FileRow {
    FileRow {
        key: key.to_owned(),
        path: format!("/etc/{key}"),
        name: key.to_owned(),
        admin,
        read_only,
    }
}

#[test]
fn the_list_counts_the_files_that_need_root() {
    let rows = [
        file("pacman.conf", true, false),
        file("os-release", true, true),
        file("fish", false, false),
    ];

    let listing = file_listing(&rows);

    assert_eq!(
        listing.summary.as_deref(),
        Some("3, 1 needs root, 1 read only")
    );
    assert_eq!(listing.head, ["FILE", "PATH", "NOTE"]);
    assert_eq!(listing.rows[0].mark, Mark::Blank);
    assert_eq!(
        listing.rows[1].cells,
        ["os-release", "/etc/os-release", "read only"]
    );
    assert_eq!(
        listing.hint.as_deref(),
        Some("garuda-toolbox configs edit <key>")
    );
}

#[test]
fn a_failed_read_keeps_the_program_s_words_out_of_the_line() {
    assert_eq!(
        read_problem(&ReadProblem::Failed("cat: Permission denied".to_owned())),
        (
            "the file was not read".to_owned(),
            "cat: Permission denied".to_owned()
        )
    );
    assert_eq!(
        read_problem(&ReadProblem::Cancelled),
        (
            "the file was not read (cancelled)".to_owned(),
            String::new()
        )
    );
}
