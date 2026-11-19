use toolbox_plan::{MergeCopy, Resolution};

use super::{Comparison, Part, Pick, Standing, edited, standing};

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

#[test]
fn each_changed_stretch_is_one_change_between_shared_lines() {
    let comparison = Comparison::new(CURRENT, COPY);

    assert_eq!(comparison.changes(), 2);
    assert_eq!(
        comparison.parts().get(1),
        Some(&Part::Change {
            current: vec!["#Color\n"],
            copy: vec!["Color\n"],
        })
    );
    assert_eq!(
        comparison.parts().get(3),
        Some(&Part::Change {
            current: vec!["ParallelDownloads = 5\n"],
            copy: vec!["ParallelDownloads = 10\n", "DownloadUser = alpm\n"],
        })
    );
}

#[test]
fn the_picks_choose_each_change_on_its_own() {
    let comparison = Comparison::new(CURRENT, COPY);
    let merged = comparison.merged(&[Pick::Take, Pick::Keep]);

    assert_eq!(merged, CURRENT.replace("#Color", "Color"));
}

#[test]
fn a_change_without_a_pick_keeps_the_current_lines() {
    let comparison = Comparison::new(CURRENT, COPY);

    assert_eq!(comparison.merged(&[]), CURRENT);
    assert_eq!(comparison.resolution(&[]).unwrap(), Resolution::KeepCurrent);
}

#[test]
fn all_changes_taken_take_the_copy_and_mixed_picks_merge() {
    let comparison = Comparison::new(CURRENT, COPY);

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
