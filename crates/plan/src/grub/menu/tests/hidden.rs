//! os-prober copies titles of other disks into `grub.cfg`, so the menu
//! is outside text.

use super::super::{GrubMenu, GrubPlatform};
use super::INSTALLED;

const FIRST_TITLE: &str = "menuentry 'Garuda Linux' ";
const FIRST_ID: &str = "'gnulinux-simple-";

#[test]
fn a_title_loses_a_bidi_override_and_a_zero_width_joiner() {
    // Changed: the captured first title gains both characters.
    let config = INSTALLED.replacen(FIRST_TITLE, "menuentry 'Garuda\u{202e} Li\u{200d}nux' ", 1);

    let menu = GrubMenu::for_platform(&config, GrubPlatform::Efi);

    assert_eq!(menu.entries[0].title, "Garuda Linux");
}

#[test]
fn an_entry_whose_id_hides_a_character_is_left_out() {
    // Changed: the captured first id gains a zero width joiner.
    let config = INSTALLED.replacen(FIRST_ID, "'gnulinux-\u{200d}simple-", 1);

    let menu = GrubMenu::for_platform(&config, GrubPlatform::Efi);

    assert!(
        menu.entries
            .iter()
            .all(|entry| !entry.id.as_str().contains('\u{200d}')),
        "{menu:?}"
    );
    assert_ne!(menu.entries[0].title, "Garuda Linux");
}
