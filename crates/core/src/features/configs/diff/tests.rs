use super::{Change, Diff, hunk_lines};

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

    assert!(diff.is_empty());
    assert_eq!((diff.added(), diff.removed()), (0, 0));
    assert_eq!(diff.plain(), "");
    assert_eq!(diff.colored(), "");
    assert!(Diff::new("", "").is_empty());
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

/// The coloured form goes through the terminal colour converter, so a
/// file's own escape sequences must not reach it as codes.
#[test]
fn the_coloured_form_marks_lines_and_defuses_the_files_control_characters() {
    let diff = Diff::new("a\n", "\u{1b}[31mred\u{1b}]8;;http://evil\u{7}\tx\n");
    let colored = diff.colored();

    assert_eq!(
        colored,
        "\u{1b}[36m@@ -1,1 +1,1 @@\u{1b}[0m\n\u{1b}[31m-a\u{1b}[0m\n\u{1b}[32m+\u{fffd}[31mred\u{fffd}]8;;http://evil\u{fffd}\tx\u{1b}[0m\n"
    );
    assert_eq!(
        diff.plain(),
        "@@ -1,1 +1,1 @@\n-a\n+\u{1b}[31mred\u{1b}]8;;http://evil\u{7}\tx\n"
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
