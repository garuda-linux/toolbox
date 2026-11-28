use toolbox_core::features::configs::Diff;

use super::Rows;

/// `/etc/default/grub` of the Garuda Lima image.
const GRUB: &str = toolbox_plan::testing::grub::LIMA_DEFAULT_GRUB;

fn rows(edited: &str) -> Rows {
    Rows::parse(&Diff::new(GRUB, edited).plain())
}

/// The capture with its timeout of 5 seconds set to 2.
fn shorter_timeout() -> String {
    GRUB.replace("GRUB_TIMEOUT=5\n", "GRUB_TIMEOUT=2\n")
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
