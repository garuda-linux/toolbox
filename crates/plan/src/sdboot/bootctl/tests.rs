use super::bootctl_default;
use crate::LoaderEntryId;
use crate::testing::sdboot::{BARE_METAL_BOOTCTL_LIST, IMAGE_BOOTCTL_LIST, SNAPSHOTS_BOOTCTL_LIST};

fn id(value: &str) -> LoaderEntryId {
    LoaderEntryId::new(value).unwrap()
}

#[test]
fn the_default_is_the_entry_bootctl_marks_on_a_bare_metal_system() {
    assert_eq!(
        bootctl_default(BARE_METAL_BOOTCTL_LIST),
        Some(id(
            "nixos-generation-235-vn3qitxasms3w4tyril3pz5323hxkh25hj66ok6l73b5lotswyzq.efi"
        ))
    );
}

#[test]
fn the_default_is_the_garuda_image_bootctl_marks() {
    assert_eq!(
        bootctl_default(IMAGE_BOOTCTL_LIST),
        Some(id("garuda-7.3.1-1-garuda.efi"))
    );
}

#[test]
fn the_default_of_the_snapshot_capture_is_the_same_garuda_image() {
    assert_eq!(
        bootctl_default(SNAPSHOTS_BOOTCTL_LIST),
        Some(id("garuda-7.3.1-1-garuda.efi"))
    );
}

/// Edge case: no entry is marked, the flag of the first entry is set to
/// `false` in this test.
#[test]
fn a_list_without_a_marked_entry_names_no_default() {
    let unmarked = BARE_METAL_BOOTCTL_LIST.replacen("\"isDefault\":true", "\"isDefault\":false", 1);

    assert_eq!(bootctl_default(&unmarked), None);
}

#[test]
fn an_answer_that_is_no_list_names_no_default() {
    assert_eq!(bootctl_default(""), None);
    assert_eq!(bootctl_default("{\"isDefault\":true}"), None);
}
