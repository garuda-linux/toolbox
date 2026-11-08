use std::collections::BTreeSet;

use super::GarudaUpdateFlag;

const SHIPPED: &str = "\
# garuda-update configuration
# SKIP_MIRRORLIST=1
UPDATE_AUR=1
PACMAN_NOCONFIRM=0
OTHER_SETTING=\"keep me\"
";

#[test]
fn keys_and_ids_name_every_flag() {
    assert_eq!(GarudaUpdateFlag::SkipMirrorlist.key(), "SKIP_MIRRORLIST");
    assert_eq!(GarudaUpdateFlag::NoCheckSpace.key(), "NO_CHECK_SPACE");

    for flag in GarudaUpdateFlag::ALL {
        assert_eq!(
            serde_json::to_string(&flag).unwrap(),
            format!("\"{}\"", flag.as_str())
        );
    }
}

#[test]
fn only_uncommented_ones_count_as_on() {
    assert_eq!(
        GarudaUpdateFlag::enabled_in(SHIPPED),
        BTreeSet::from([GarudaUpdateFlag::UpdateAur])
    );
    assert_eq!(GarudaUpdateFlag::enabled_in(""), BTreeSet::new());
}

#[test]
fn quotes_spaces_and_trailing_comments_are_understood() {
    let config = "  NO_CHECK_SPACE=\"1\"  # keep\nPACMAN_NOCONFIRM='1'\nSKIP_MIRRORLIST=yes\n";

    assert_eq!(
        GarudaUpdateFlag::enabled_in(config),
        BTreeSet::from([
            GarudaUpdateFlag::PacmanNoconfirm,
            GarudaUpdateFlag::NoCheckSpace
        ])
    );
}

#[test]
fn the_last_assignment_wins_like_in_the_shell() {
    assert!(!GarudaUpdateFlag::UpdateAur.is_on("UPDATE_AUR=1\nUPDATE_AUR=0\n"));
    assert!(GarudaUpdateFlag::UpdateAur.is_on("UPDATE_AUR=0\nUPDATE_AUR=1\n"));
}

#[test]
fn a_similar_key_is_a_different_switch() {
    assert!(!GarudaUpdateFlag::UpdateAur.is_on("UPDATE_AUR_EXTRA=1\n"));
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
    assert_eq!(
        GarudaUpdateFlag::PacmanNoconfirm.set(SHIPPED, true),
        SHIPPED.replace("PACMAN_NOCONFIRM=0", "PACMAN_NOCONFIRM=1")
    );
}

#[test]
fn switching_on_appends_a_missing_key() {
    assert_eq!(
        GarudaUpdateFlag::NoCheckSpace.set(SHIPPED, true),
        format!("{SHIPPED}NO_CHECK_SPACE=1\n")
    );
    assert_eq!(
        GarudaUpdateFlag::NoCheckSpace.set("A=1", true),
        "A=1\nNO_CHECK_SPACE=1\n"
    );
    assert_eq!(
        GarudaUpdateFlag::NoCheckSpace.set("", true),
        "NO_CHECK_SPACE=1\n"
    );
}

#[test]
fn switching_off_comments_every_assignment_out() {
    let config = "UPDATE_AUR=1\r\nX=2\nUPDATE_AUR=1";

    assert_eq!(
        GarudaUpdateFlag::UpdateAur.set(config, false),
        "# UPDATE_AUR=1\r\nX=2\n# UPDATE_AUR=1"
    );
}

#[test]
fn switching_off_what_is_not_set_changes_nothing() {
    assert_eq!(
        GarudaUpdateFlag::SkipMirrorlist.set(SHIPPED, false),
        SHIPPED
    );
}

#[test]
fn a_written_switch_reads_back() {
    for flag in GarudaUpdateFlag::ALL {
        for on in [true, false] {
            assert_eq!(flag.is_on(&flag.set(SHIPPED, on)), on, "{flag:?} {on}");
        }
    }
}
