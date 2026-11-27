use proptest::prelude::*;
use toolbox_plan::{ConfigText, MergeCopy, Resolution};

use super::{
    CachedComparison, Comparison, Part, Pick, Standing, edit_start, edited, settled, standing,
};

/// Lines in the style of `/etc/pacman.conf`, and a copy that switches
/// `Color` on and changes the last line into two.
const CURRENT: &str = "\
[options]
HoldPkg     = pacman glibc
Architecture = auto
#Color
CheckSpace
ParallelDownloads = 5
";
const COPY: &str = "\
[options]
HoldPkg     = pacman glibc
Architecture = auto
Color
CheckSpace
ParallelDownloads = 10
DownloadUser = alpm
";

fn copy(path: &str) -> MergeCopy {
    MergeCopy::new(path).unwrap()
}

fn compare(current: &str, copy: &str) -> Comparison {
    Comparison::new(current.to_owned(), copy.to_owned())
}

#[test]
fn each_changed_stretch_is_one_change_between_shared_lines() {
    let comparison = compare(CURRENT, COPY);

    assert_eq!(comparison.changes(), 2);
    assert_eq!(
        comparison.parts().nth(1),
        Some(Part::Change {
            current: "#Color\n",
            copy: "Color\n",
        })
    );
    assert_eq!(
        comparison.parts().nth(3),
        Some(Part::Change {
            current: "ParallelDownloads = 5\n",
            copy: "ParallelDownloads = 10\nDownloadUser = alpm\n",
        })
    );
}

#[test]
fn the_picks_choose_each_change_on_its_own() {
    let comparison = compare(CURRENT, COPY);
    let merged = comparison.merged(&[Pick::Take, Pick::Keep]);

    assert_eq!(merged, CURRENT.replace("#Color", "Color"));
}

#[test]
fn a_change_without_a_pick_keeps_the_current_lines() {
    let comparison = compare(CURRENT, COPY);

    assert_eq!(comparison.merged(&[]), CURRENT);
    assert_eq!(comparison.resolution(&[]).unwrap(), Resolution::KeepCurrent);
}

#[test]
fn all_changes_taken_take_the_copy_and_mixed_picks_merge() {
    let comparison = compare(CURRENT, COPY);

    assert_eq!(
        comparison.resolution(&[Pick::Take, Pick::Take]).unwrap(),
        Resolution::TakeCopy
    );
    assert!(matches!(
        comparison.resolution(&[Pick::Keep, Pick::Take]).unwrap(),
        Resolution::Merged { text } if text.as_str() == COPY.replace("Color\n", "#Color\n")
    ));
}

#[test]
fn a_text_edited_by_hand_that_equals_a_side_is_that_side() {
    assert_eq!(
        edited(CURRENT, CURRENT, COPY).unwrap(),
        Resolution::KeepCurrent
    );
    assert_eq!(edited(COPY, CURRENT, COPY).unwrap(), Resolution::TakeCopy);
    assert!(matches!(
        edited("[options]\n", CURRENT, COPY).unwrap(),
        Resolution::Merged { text: _ }
    ));
}

#[test]
fn the_editor_starts_from_the_picks() {
    let comparison = compare(CURRENT, COPY);
    let start = |picks: &[Pick]| edit_start(&comparison.resolution(picks), Some(CURRENT), COPY);

    assert_eq!(start(&[Pick::Take, Pick::Take]), COPY);
    assert_eq!(start(&[Pick::Keep, Pick::Keep]), CURRENT);
    assert_eq!(
        start(&[Pick::Take, Pick::Keep]),
        CURRENT.replace("#Color", "Color")
    );
}

#[test]
fn a_merge_that_is_no_config_text_starts_the_editor_from_the_current_file() {
    let refused = ConfigText::new("a\0b").map(|text| Resolution::Merged { text });

    assert!(refused.is_err());
    assert_eq!(edit_start(&refused, Some(CURRENT), COPY), CURRENT);
}

#[test]
fn a_row_says_how_the_copy_stands() {
    let pacnew = copy("/etc/pacman.conf.pacnew");

    assert_eq!(
        standing(&pacnew, Some(CURRENT), COPY),
        Standing::Differs { changes: 2 }
    );
    assert_eq!(standing(&pacnew, Some(COPY), COPY), Standing::Same);
    assert_eq!(standing(&pacnew, None, COPY), Standing::CurrentGone);
    assert_eq!(
        standing(&copy("/etc/pacman.conf.pacsave.1"), Some(CURRENT), COPY),
        Standing::OlderSave
    );
}

/// Parts as `(changed, current, copy)`.
fn shape(comparison: &Comparison) -> Vec<(bool, String, String)> {
    comparison
        .parts()
        .map(|part| match part {
            Part::Same(lines) => (false, lines.to_owned(), lines.to_owned()),
            Part::Change { current, copy } => (true, current.to_owned(), copy.to_owned()),
        })
        .collect()
}

fn same(text: &str) -> (bool, String, String) {
    (false, text.to_owned(), text.to_owned())
}

fn change(current: &str, copy: &str) -> (bool, String, String) {
    (true, current.to_owned(), copy.to_owned())
}

fn compared(current: &str, copy: &str) -> (Vec<(bool, String, String)>, usize, String, Resolution) {
    let comparison = compare(current, copy);
    let picks = [Pick::Take, Pick::Keep];

    (
        shape(&comparison),
        comparison.changes(),
        comparison.merged(&picks),
        comparison.resolution(&picks).unwrap(),
    )
}

fn merged(text: &str) -> Resolution {
    Resolution::Merged {
        text: ConfigText::new(text).unwrap(),
    }
}

#[test]
fn the_parts_of_the_pacman_lines_hold_whole_lines() {
    assert_eq!(
        compared(CURRENT, COPY),
        (
            vec![
                same("[options]\nHoldPkg     = pacman glibc\nArchitecture = auto\n"),
                change("#Color\n", "Color\n"),
                same("CheckSpace\n"),
                change(
                    "ParallelDownloads = 5\n",
                    "ParallelDownloads = 10\nDownloadUser = alpm\n"
                ),
            ],
            2,
            CURRENT.replace("#Color", "Color"),
            merged(&CURRENT.replace("#Color", "Color")),
        )
    );
}

#[test]
fn crlf_lines_keep_their_line_breaks() {
    assert_eq!(
        compared("a\r\nb\r\nc\r\n", "A\r\nb\r\nc\n"),
        (
            vec![
                change("a\r\n", "A\r\n"),
                same("b\r\n"),
                change("c\r\n", "c\n")
            ],
            2,
            "A\r\nb\r\nc\r\n".to_owned(),
            merged("A\r\nb\r\nc\r\n"),
        )
    );
}

#[test]
fn a_missing_final_line_break_is_a_change_of_the_last_line() {
    assert_eq!(
        compared("a\nb\nc", "x\nb\nc\n"),
        (
            vec![change("a\n", "x\n"), same("b\n"), change("c", "c\n")],
            2,
            "x\nb\nc".to_owned(),
            merged("x\nb\nc"),
        )
    );
}

#[test]
fn an_empty_file_is_one_change_against_the_whole_copy() {
    assert_eq!(
        compared("", "a\nb\n"),
        (
            vec![change("", "a\nb\n")],
            1,
            "a\nb\n".to_owned(),
            Resolution::TakeCopy,
        )
    );
    assert_eq!(
        compared("a\n", ""),
        (
            vec![change("a\n", "")],
            1,
            String::new(),
            Resolution::TakeCopy,
        )
    );
}

#[test]
fn multi_byte_lines_stay_whole() {
    assert_eq!(
        compared("Grüße\nÄ\nmitte\n日本\n", "Grüße\nÖ\nmitte\n日本語\n"),
        (
            vec![
                same("Grüße\n"),
                change("Ä\n", "Ö\n"),
                same("mitte\n"),
                change("日本\n", "日本語\n")
            ],
            2,
            "Grüße\nÖ\nmitte\n日本\n".to_owned(),
            merged("Grüße\nÖ\nmitte\n日本\n"),
        )
    );
}

#[test]
fn only_differing_lines_need_a_comparison() {
    let pacnew = copy("/etc/pacman.conf.pacnew");

    assert_eq!(settled(&pacnew, Some(CURRENT), COPY), None);
    assert_eq!(settled(&pacnew, Some(COPY), COPY), Some(Standing::Same));
    assert_eq!(settled(&pacnew, None, COPY), Some(Standing::CurrentGone));
}

#[test]
fn a_comparison_holds_exactly_its_texts() {
    let comparison = compare(CURRENT, COPY);

    assert!(comparison.holds(CURRENT, COPY));
    assert!(!comparison.holds(COPY, CURRENT));
    assert!(!comparison.holds(&CURRENT.replace('\n', "\r\n"), COPY));
}

fn merged_text(resolution: &Result<Resolution, toolbox_plan::InvalidConfigText>) -> *const u8 {
    match resolution {
        Ok(Resolution::Merged { text }) => text.as_str().as_ptr(),
        other => panic!("no merged text: {other:?}"),
    }
}

/// Only new picks build the text again.
#[test]
fn the_cached_resolution_follows_the_picks() {
    let mut cached = CachedComparison::new(CURRENT.to_owned(), COPY.to_owned());

    assert_eq!(cached.resolution(&[]), &Ok(Resolution::KeepCurrent));

    let first = merged_text(cached.resolution(&[Pick::Take, Pick::Keep]));
    assert_eq!(
        merged_text(cached.resolution(&[Pick::Take, Pick::Keep])),
        first
    );

    assert_eq!(
        cached.resolution(&[Pick::Take, Pick::Take]),
        &Ok(Resolution::TakeCopy)
    );
    assert_eq!(
        cached.resolution(&[Pick::Take, Pick::Keep]),
        &compare(CURRENT, COPY).resolution(&[Pick::Take, Pick::Keep])
    );
    assert_eq!(cached.comparison(), &compare(CURRENT, COPY));
}

/// Texts of short lines, with multi-byte characters, CRLF breaks and a line
/// without its break at the end.
fn text() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop::sample::select(vec!["a\n", "b\n", "\u{fc}\n", "\n", "\u{e9}\r\n", "x"]),
        0..12,
    )
    .prop_map(|lines| lines.concat())
}

proptest! {
    #[test]
    fn the_parts_rebuild_both_texts_exactly(current in text(), copy in text()) {
        let comparison = Comparison::new(current.clone(), copy.clone());
        let (mut from_current, mut from_copy) = (String::new(), String::new());

        for part in comparison.parts() {
            match part {
                Part::Same(lines) => {
                    from_current.push_str(lines);
                    from_copy.push_str(lines);
                }
                Part::Change {
                    current: now,
                    copy: taken,
                } => {
                    from_current.push_str(now);
                    from_copy.push_str(taken);
                }
            }
        }

        prop_assert_eq!(from_current, current);
        prop_assert_eq!(from_copy, copy);
    }

    #[test]
    fn every_change_picked_one_way_merges_to_one_side(current in text(), copy in text()) {
        let comparison = Comparison::new(current.clone(), copy.clone());
        let changes = comparison.changes();

        prop_assert_eq!(comparison.merged(&vec![Pick::Keep; changes]), current);
        prop_assert_eq!(comparison.merged(&vec![Pick::Take; changes]), copy);
    }
}
