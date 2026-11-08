use std::collections::BTreeSet;

use super::GarudaUpdateFlag;

/// `/etc/garuda/garuda-update/config` as garuda-update ships it, every
/// variable commented out and no line break at the end.
const SHIPPED: &str = include_str!("../../../system/src/settings/fixtures/garuda-update-config");

#[test]
fn keys_and_ids_name_every_flag() {
    assert_eq!(GarudaUpdateFlag::SkipMirrorlist.key(), "SKIP_MIRRORLIST");
    assert_eq!(GarudaUpdateFlag::NoCheckSpace.key(), "NO_SPACE_CHECK");

    for flag in GarudaUpdateFlag::ALL {
        assert_eq!(
            serde_json::to_string(&flag).unwrap(),
            format!("\"{}\"", flag.as_str())
        );
    }
}

#[test]
fn only_uncommented_ones_count_as_on() {
    assert_eq!(GarudaUpdateFlag::enabled_in(SHIPPED), BTreeSet::new());

    // Edit: the shipped `UPDATE_AUR` line uncommented.
    assert_eq!(
        GarudaUpdateFlag::enabled_in(&SHIPPED.replace("# UPDATE_AUR=1", "UPDATE_AUR=1")),
        BTreeSet::from([GarudaUpdateFlag::UpdateAur])
    );
    assert_eq!(GarudaUpdateFlag::enabled_in(""), BTreeSet::new());
}

#[test]
fn the_space_check_is_written_under_the_shipped_name() {
    assert_eq!(
        GarudaUpdateFlag::NoCheckSpace.set(SHIPPED, true),
        SHIPPED.replace("# NO_SPACE_CHECK=1", "NO_SPACE_CHECK=1")
    );
}

/// garuda-update reads `NO_CHECK_SPACE`, the name older toolbox versions
/// wrote, only when `NO_SPACE_CHECK` is unset.
#[test]
fn the_old_space_check_name_counts_unless_the_new_one_is_set() {
    // Edit: the old name appended to the shipped file.
    let old = format!("{SHIPPED}\nNO_CHECK_SPACE=1\n");

    assert!(GarudaUpdateFlag::NoCheckSpace.is_on(&old));

    // Edit: the shipped line uncommented as 0 and the old name appended.
    let both = format!(
        "{}\nNO_CHECK_SPACE=1\n",
        SHIPPED.replace("# NO_SPACE_CHECK=1", "NO_SPACE_CHECK=0")
    );

    assert!(!GarudaUpdateFlag::NoCheckSpace.is_on(&both));
}

#[test]
fn switching_the_space_check_off_comments_the_old_name_out_too() {
    // Edit: the old name appended to the shipped file.
    let old = format!("{SHIPPED}\nNO_CHECK_SPACE=1\n");

    assert_eq!(
        GarudaUpdateFlag::NoCheckSpace.set(&old, false),
        format!("{SHIPPED}\n# NO_CHECK_SPACE=1\n")
    );
}

#[test]
fn quotes_spaces_and_trailing_comments_are_understood() {
    // Edit: three shipped lines uncommented with quotes, indent, a trailing
    // comment and a value that is no `1`.
    let config = SHIPPED
        .replace("# PACMAN_NOCONFIRM=1", "PACMAN_NOCONFIRM='1'")
        .replace("# SKIP_MIRRORLIST=1", "SKIP_MIRRORLIST=yes")
        .replace("# UPDATE_AUR=1", "  UPDATE_AUR=\"1\"  # keep");

    assert_eq!(
        GarudaUpdateFlag::enabled_in(&config),
        BTreeSet::from([
            GarudaUpdateFlag::UpdateAur,
            GarudaUpdateFlag::PacmanNoconfirm
        ])
    );
}

#[test]
fn the_last_assignment_wins_like_in_the_shell() {
    // Edit: two assignments appended to the shipped file.
    let off = format!("{SHIPPED}\nUPDATE_AUR=1\nUPDATE_AUR=0\n");
    let on = format!("{SHIPPED}\nUPDATE_AUR=0\nUPDATE_AUR=1\n");

    assert!(!GarudaUpdateFlag::UpdateAur.is_on(&off));
    assert!(GarudaUpdateFlag::UpdateAur.is_on(&on));
}

#[test]
fn a_similar_key_is_a_different_switch() {
    // Edit: a longer key appended to the shipped file.
    assert!(!GarudaUpdateFlag::UpdateAur.is_on(&format!("{SHIPPED}\nUPDATE_AUR_EXTRA=1\n")));
}

#[test]
fn switching_on_revives_the_commented_line_in_place() {
    assert_eq!(
        GarudaUpdateFlag::SkipMirrorlist.set(SHIPPED, true),
        SHIPPED.replace("# SKIP_MIRRORLIST=1", "SKIP_MIRRORLIST=1")
    );
}

#[test]
fn switching_on_rewrites_an_existing_assignment() {
    // Edit: the shipped line set to 0.
    let config = SHIPPED.replace("# PACMAN_NOCONFIRM=1", "PACMAN_NOCONFIRM=0");

    assert_eq!(
        GarudaUpdateFlag::PacmanNoconfirm.set(&config, true),
        SHIPPED.replace("# PACMAN_NOCONFIRM=1", "PACMAN_NOCONFIRM=1")
    );
}

#[test]
fn switching_on_appends_a_missing_key() {
    assert_eq!(GarudaUpdateFlag::UpdateAur.set("", true), "UPDATE_AUR=1\n");

    // Edit: the shipped file with a line break at its end and its
    // `UPDATE_AUR` line removed.
    let without = format!("{SHIPPED}\n").replace("# UPDATE_AUR=1\n", "");

    assert_eq!(
        GarudaUpdateFlag::UpdateAur.set(&without, true),
        format!("{without}UPDATE_AUR=1\n")
    );
}

#[test]
fn switching_off_comments_every_assignment_out() {
    // Edit: the shipped line uncommented with CRLF, and once more without
    // a line break at the end.
    let config = format!(
        "{}\nUPDATE_AUR=1",
        SHIPPED.replace("# UPDATE_AUR=1\n", "UPDATE_AUR=1\r\n")
    );

    assert_eq!(
        GarudaUpdateFlag::UpdateAur.set(&config, false),
        format!(
            "{}\n# UPDATE_AUR=1",
            SHIPPED.replace("# UPDATE_AUR=1\n", "# UPDATE_AUR=1\r\n")
        )
    );
}

#[test]
fn switching_off_what_is_not_set_changes_nothing() {
    for flag in GarudaUpdateFlag::ALL {
        assert_eq!(flag.set(SHIPPED, false), SHIPPED, "{flag:?}");
    }
}

#[test]
fn a_written_switch_reads_back() {
    for flag in GarudaUpdateFlag::ALL {
        for on in [true, false] {
            assert_eq!(flag.is_on(&flag.set(SHIPPED, on)), on, "{flag:?} {on}");
        }
    }
}
