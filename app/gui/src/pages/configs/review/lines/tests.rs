use toolbox_core::features::configs::Diff;

use super::{Covered, Kind, Row, Rows};

/// `/etc/default/grub` of the Garuda Lima image.
const GRUB: &str = toolbox_plan::testing::grub::LIMA_DEFAULT_GRUB;

fn rows(edited: &str) -> Rows {
    Rows::parse(&Diff::new(GRUB, edited).plain())
}

fn find(rows: &Rows, kind: Kind) -> &Row {
    rows.lines.iter().find(|row| row.kind == kind).unwrap()
}

/// The capture with its timeout of 5 seconds set to 2.
fn shorter_timeout() -> String {
    GRUB.replace("GRUB_TIMEOUT=5\n", "GRUB_TIMEOUT=2\n")
}

#[test]
fn a_changed_line_has_its_number_in_both_files() {
    let rows = rows(&shorter_timeout());

    assert_eq!(
        rows.lines.first().map(|row| row.kind),
        Some(Kind::Hunk(Covered { first: 1, last: 7 }))
    );

    let removed = find(&rows, Kind::Removed);
    assert_eq!((removed.old, removed.new), (Some(4), None));
    assert_eq!(removed.text(), "GRUB_TIMEOUT=5");
    assert_eq!(removed.line, "-GRUB_TIMEOUT=5");

    let added = find(&rows, Kind::Added);
    assert_eq!((added.old, added.new), (None, Some(4)));
    assert_eq!(added.text(), "GRUB_TIMEOUT=2");

    let after = rows
        .lines
        .iter()
        .skip_while(|row| row.kind != Kind::Added)
        .nth(1)
        .unwrap();

    assert_eq!(
        (after.kind, after.old, after.new),
        (Kind::Same, Some(5), Some(5))
    );
}

/// The capture with a line added after its timeout.
#[test]
fn an_added_line_moves_the_numbers_of_the_edited_file_on() {
    let edited = GRUB.replace(
        "GRUB_TIMEOUT=5\n",
        "GRUB_TIMEOUT=5\nGRUB_RECORDFAIL_TIMEOUT=5\n",
    );

    let rows = rows(&edited);

    let after = rows
        .lines
        .iter()
        .skip_while(|row| row.kind != Kind::Added)
        .nth(1)
        .unwrap();

    assert_eq!(
        (after.kind, after.old, after.new),
        (Kind::Same, Some(5), Some(6))
    );
}

/// The capture with two tabs and a comment after its timeout, longer than
/// the lines around it.
#[test]
fn a_tab_counts_as_the_spaces_egui_lays_it_out_with() {
    let line = "GRUB_TIMEOUT=2\t\t# wait two seconds, then start the default entry";
    let edited = GRUB.replace("GRUB_TIMEOUT=5\n", &format!("{line}\n"));
    let around = GRUB
        .lines()
        .take(7)
        .map(|text| text.chars().count())
        .max()
        .unwrap();

    let tabbed = line.chars().count() + 2 * (super::TAB_COLUMNS - 1);

    assert!(tabbed > around, "the tabbed line is the widest");

    let rows = rows(&edited);

    assert_eq!(usize::from(rows.columns), tabbed);
}

/// The capture with its last line changed: the numbers reach two digits.
#[test]
fn the_gutter_is_as_wide_as_the_highest_number() {
    assert_eq!(rows(&shorter_timeout()).digits, 1);

    let last = GRUB.lines().last().unwrap();
    let edited = GRUB.replace(last, &format!("{last} "));

    assert_eq!(rows(&edited).digits, 2);
}

/// The capture without its last line break: diff's note is a row without
/// numbers.
#[test]
fn the_missing_line_break_is_a_note_without_numbers() {
    let edited = GRUB.strip_suffix('\n').unwrap();
    let rows = rows(edited);

    let note = rows.lines.last().unwrap();
    assert_eq!((note.kind, note.old, note.new), (Kind::Note, None, None));
}

#[test]
fn rows_are_split_once_until_the_diff_changes() {
    let ctx = egui::Context::default();
    let plain = Diff::new(GRUB, &shorter_timeout()).plain();

    let first = super::cached(&ctx, &plain);
    let again = super::cached(&ctx, &plain);
    assert!(std::sync::Arc::ptr_eq(&first, &again));

    let other = Diff::new(GRUB, GRUB.strip_suffix('\n').unwrap()).plain();
    let changed = super::cached(&ctx, &other);
    assert!(!std::sync::Arc::ptr_eq(&first, &changed));
}
