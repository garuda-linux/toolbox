use toolbox_plan::ConfigText;

use super::{Change, Covered, Diff, Kind, Row, hunk_lines, parse_rows};
use crate::features::configs::session::Compared;

/// `/etc/default/grub` of the Garuda Lima image.
const GRUB: &str = toolbox_plan::testing::grub::LIMA_DEFAULT_GRUB;

fn is_empty(diff: &Diff<'_>) -> bool {
    diff.lines().iter().all(|line| line.change == Change::Same)
}

/// One line per number: `<word> <number>`.
fn numbered(word: &str, numbers: std::ops::Range<u32>) -> String {
    numbers
        .map(|number| format!("{word} {number}\n"))
        .collect::<Vec<_>>()
        .concat()
}

fn changes(diff: &Diff<'_>) -> Vec<(Change, String)> {
    diff.lines()
        .iter()
        .map(|line| (line.change, line.text.to_owned()))
        .collect()
}

#[test]
fn equal_texts_have_no_changes() {
    let diff = Diff::new("a\nb\n", "a\nb\n");

    assert!(is_empty(&diff));
    assert_eq!((diff.added(), diff.removed()), (0, 0));
    assert_eq!(diff.plain(), "");
    assert!(is_empty(&Diff::new("", "")));
}

#[test]
fn a_changed_line_is_removed_and_added() {
    let diff = Diff::new("a\nb\nc\n", "a\nB\nc\n");

    assert_eq!(
        changes(&diff),
        [
            (Change::Same, "a\n".to_owned()),
            (Change::Removed, "b\n".to_owned()),
            (Change::Added, "B\n".to_owned()),
            (Change::Same, "c\n".to_owned()),
        ]
    );
    assert_eq!((diff.added(), diff.removed()), (1, 1));
    assert_eq!(diff.plain(), "@@ -1,3 +1,3 @@\n a\n-b\n+B\n c\n");
}

#[test]
fn inserted_and_deleted_lines_keep_what_both_texts_share() {
    let diff = Diff::new("one\ntwo\nthree\nfour\n", "one\nthree\nnew\nfour\nfive\n");

    assert_eq!(
        diff.plain(),
        "@@ -1,4 +1,5 @@\n one\n-two\n three\n+new\n four\n+five\n"
    );
    assert_eq!(Diff::new("", "a\nb\n").plain(), "@@ -1,0 +1,2 @@\n+a\n+b\n");
    assert_eq!(Diff::new("a\n", "").plain(), "@@ -1,1 +1,0 @@\n-a\n");
}

#[test]
fn far_apart_changes_show_as_hunks_with_three_lines_around() {
    let old = numbered("line", 1..31);
    let new = old
        .replace("line 5\n", "line five\n")
        .replace("line 25\n", "");

    assert_eq!(
        Diff::new(&old, &new).plain(),
        "@@ -2,7 +2,7 @@\n line 2\n line 3\n line 4\n-line 5\n+line five\n line 6\n line 7\n line 8\n\
         @@ -22,7 +22,6 @@\n line 22\n line 23\n line 24\n-line 25\n line 26\n line 27\n line 28\n"
    );
}

#[test]
fn a_missing_last_line_break_is_said() {
    assert_eq!(
        Diff::new("a\nb\n", "a\nb").plain(),
        "@@ -1,2 +1,2 @@\n a\n-b\n+b\n\\ No line break at the end of the file\n"
    );
    assert_eq!(
        Diff::new("a", "a\n").plain(),
        "@@ -1,1 +1,1 @@\n-a\n\\ No line break at the end of the file\n+a\n"
    );
}

#[test]
fn a_huge_change_is_still_a_correct_diff() {
    let old = numbered("old", 0..3000);
    let new = numbered("new", 0..3000);
    let diff = Diff::new(&old, &new);

    assert_eq!((diff.removed(), diff.added()), (3000, 3000));
    assert!(diff.plain().starts_with("@@ -1,3000 +1,3000 @@\n-old 0\n"));
}

#[test]
fn a_hunk_header_names_the_lines_of_the_new_text() {
    assert_eq!(hunk_lines("@@ -3,3 +3,4 @@"), Some((3, 6)));
    assert_eq!(hunk_lines("@@ -10,1 +10,1 @@"), Some((10, 10)));
}

#[test]
fn a_hunk_that_only_removes_names_the_line_it_follows() {
    assert_eq!(hunk_lines("@@ -5,2 +4,0 @@"), Some((4, 4)));
}

#[test]
fn a_changed_line_is_no_hunk_header() {
    assert_eq!(hunk_lines("+GRUB_TIMEOUT=5"), None);
    assert_eq!(hunk_lines("@@ broken @@"), None);
}

#[test]
fn a_hidden_character_of_an_edit_stays_out_of_the_changes() {
    let edited = ConfigText::new("a = \u{202e}2\u{200b}\n".to_owned()).unwrap();
    let original = ConfigText::new("a = 1\n".to_owned()).unwrap();

    let Compared::Changed(changes) = Compared::between(original, edited) else {
        panic!("the texts differ");
    };

    assert_eq!(changes.plain, "@@ -1,1 +1,1 @@\n-a = 1\n+a = 2\n");
    assert_eq!(changes.edited.as_str(), "a = \u{202e}2\u{200b}\n");
}

/// The capture with its timeout of 5 seconds set to 2.
fn shorter_timeout() -> String {
    GRUB.replace("GRUB_TIMEOUT=5\n", "GRUB_TIMEOUT=2\n")
}

fn rows_of(edited: &str) -> Vec<Row> {
    parse_rows(&Diff::new(GRUB, edited).plain())
}

fn find(rows: &[Row], kind: Kind) -> &Row {
    rows.iter().find(|row| row.kind == kind).unwrap()
}

fn line_after_added(rows: &[Row]) -> &Row {
    rows.iter()
        .skip_while(|row| row.kind != Kind::Added)
        .nth(1)
        .unwrap()
}

#[test]
fn a_changed_line_has_its_number_in_both_files() {
    let rows = rows_of(&shorter_timeout());

    assert_eq!(
        rows.first().map(|row| row.kind),
        Some(Kind::Hunk(Covered { first: 1, last: 7 }))
    );

    let removed = find(&rows, Kind::Removed);
    assert_eq!((removed.old, removed.new), (Some(4), None));
    assert_eq!(removed.text(), "GRUB_TIMEOUT=5");
    assert_eq!(removed.line, "-GRUB_TIMEOUT=5");

    let added = find(&rows, Kind::Added);
    assert_eq!((added.old, added.new), (None, Some(4)));
    assert_eq!(added.text(), "GRUB_TIMEOUT=2");

    let after = line_after_added(&rows);

    assert_eq!(
        (after.kind, after.old, after.new),
        (Kind::Same, Some(5), Some(5))
    );
}

#[test]
fn an_added_line_moves_the_numbers_of_the_edited_file_on() {
    let edited = GRUB.replace(
        "GRUB_TIMEOUT=5\n",
        "GRUB_TIMEOUT=5\nGRUB_RECORDFAIL_TIMEOUT=5\n",
    );

    let rows = rows_of(&edited);
    let after = line_after_added(&rows);

    assert_eq!(
        (after.kind, after.old, after.new),
        (Kind::Same, Some(5), Some(6))
    );
}

#[test]
fn the_missing_line_break_is_a_note_without_numbers() {
    let edited = GRUB.strip_suffix('\n').unwrap();
    let rows = rows_of(edited);

    let note = rows.last().unwrap();
    assert_eq!((note.kind, note.old, note.new), (Kind::Note, None, None));
}
