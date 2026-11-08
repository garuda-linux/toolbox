use super::sdboot::{CMDLINE, LIVE, image};
use crate::features::boot::view::{Page, parameters_line, parameters_queued, plymouth_shown};
use crate::features::boot::{Flag, remove_parameter, set_flag, set_plymouth_theme};
use crate::testing::plymouth_theme;
use crate::{ChangeSet, Toggle};

/// The switches own `quiet`, so the chips show the rest in order.
#[test]
fn the_kernel_cmdline_fills_the_chips_and_the_switches() {
    let page = Page::read(&image(CMDLINE, &[]), &ChangeSet::default());

    assert!(page.parameters_editable());
    assert_eq!(
        page.parameter_items(),
        [
            "root=UUID=cd2fff21-4bb7-40f3-a158-406f545e11e6",
            "rootflags=subvol=@",
            "rw",
            "loglevel=3",
            "console=tty0",
            "console=ttyS0,115200",
        ]
    );
    assert!(page.flag(Flag::Quiet));
    assert!(!page.flag(Flag::Splash));
}

#[test]
fn the_root_chips_have_no_remove_button() {
    let page = Page::read(&image(CMDLINE, &[]), &ChangeSet::default());

    assert!(!page.removable("root=UUID=cd2fff21-4bb7-40f3-a158-406f545e11e6"));
    assert!(!page.removable("rootflags=subvol=@"));
    assert!(!page.removable("rw"));
    assert!(page.removable("loglevel=3"));
}

#[test]
fn a_queued_change_shows_and_marks_its_switch() {
    let snapshot = image(CMDLINE, &[]);
    let mut changes = ChangeSet::default();

    set_flag(Flag::Quiet, Toggle::Off, &mut changes, &snapshot).unwrap();
    remove_parameter("loglevel=3", &mut changes, &snapshot).unwrap();

    let page = Page::read(&snapshot, &changes);

    assert!(!page.flag(Flag::Quiet));
    assert_eq!(page.flags_queued(), ["quiet"]);
    assert!(
        !page
            .parameter_items()
            .iter()
            .any(|item| item == "loglevel=3")
    );
    assert!(parameters_queued(&changes));
}

#[test]
fn a_line_the_page_cannot_write_back_is_shown_as_text() {
    let snapshot = image(LIVE, &[]);
    let page = Page::read(&snapshot, &ChangeSet::default());

    assert!(!page.parameters_editable());
    assert!(parameters_line(&snapshot).starts_with("initrd=\\garuda\\amd-ucode.img "));
}

#[test]
fn a_queued_theme_shows_instead_of_the_current_one() {
    let snapshot = image(CMDLINE, &["plymouth"]);
    let mut changes = ChangeSet::default();

    assert_eq!(plymouth_shown(&snapshot, &changes), "bgrt");
    set_plymouth_theme(plymouth_theme("spinner"), &mut changes, &snapshot).unwrap();
    assert_eq!(plymouth_shown(&snapshot, &changes), "spinner");
}
