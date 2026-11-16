use super::{EspPlace, esp_place};

/// `/proc/self/mountinfo` of the Garuda Lima image.
const IMAGE: &str = crate::testing::sdboot::MOUNTINFO;

#[test]
fn the_images_esp_is_at_efi() {
    assert_eq!(esp_place(IMAGE), Some(EspPlace::Efi));
}

/// The image's ESP line, moved to the other places `bootctl` looks.
#[test]
fn the_esp_is_found_at_the_other_places_too() {
    for (place, expected) in [("/boot", EspPlace::Boot), ("/boot/efi", EspPlace::BootEfi)] {
        let moved = IMAGE.replace(" /efi ", &format!(" {place} "));

        assert_eq!(esp_place(&moved), Some(expected), "{place}");
    }
}

/// The image's ESP line as an automount that nothing opened yet.
#[test]
fn an_automount_is_no_esp_yet() {
    let automount = IMAGE.replace("- vfat /dev/vda1", "- autofs systemd-1");

    assert_eq!(esp_place(&automount), None);
}
