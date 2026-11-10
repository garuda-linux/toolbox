use toolbox_plan::{GrubPicture, GrubSetting};

use super::super::{PictureSource, set, shown};
use super::system;
use crate::ChangeSet;

/// `/etc/default/grub` of the Lima VM, with Garuda's own GRUB theme.
const LIMA: &str = toolbox_plan::testing::grub::LIMA_DEFAULT_GRUB;
/// `/etc/default/grub` of a Garuda Mokka install, with no picture.
const MOKKA: &str = toolbox_plan::testing::grub::INSTALLED_DEFAULT_GRUB;
const THEME_PICTURE: &str = "/usr/share/grub/themes/garuda/background.png";
const WALLPAPER_LINE: &str = "#GRUB_BACKGROUND=\"/path/to/wallpaper\"";

/// Edit: [`LIMA`] with an `export GRUB_MENU_PICTURE` line appended, the
/// shape garuda-boot-options writes. No captured file holds one.
fn with_menu_picture() -> String {
    format!("{LIMA}export GRUB_MENU_PICTURE=\"{THEME_PICTURE}\"\n")
}

/// `defaults` with its commented wallpaper line made an own picture.
fn with_own_picture(defaults: &str) -> String {
    let edited = defaults.replace(WALLPAPER_LINE, "GRUB_BACKGROUND=\"/boot/own.png\"");

    assert_ne!(edited, defaults);
    edited
}

fn picture(path: &str) -> GrubPicture {
    GrubPicture::new(path).unwrap()
}

#[test]
fn the_real_garuda_file_shows_no_picture() {
    for defaults in [LIMA, MOKKA] {
        let page = shown(&system(defaults, &[]), &ChangeSet::default()).unwrap();

        assert_eq!(page.background, "");
        assert_eq!(page.background_source, PictureSource::Own);
    }
}

#[test]
fn a_menu_picture_line_shows_the_picture_of_its_theme_and_not_none() {
    let state = system(&with_menu_picture(), &[]);
    let page = shown(&state, &ChangeSet::default()).unwrap();

    assert_eq!(page.background, THEME_PICTURE);
    assert_eq!(page.background_source, PictureSource::Theme);
}

#[test]
fn an_own_picture_queues_and_replaces_the_one_of_the_theme_on_the_page() {
    let state = system(&with_menu_picture(), &[]);
    let mut changes = ChangeSet::default();

    set(
        GrubSetting::Background(Some(picture("/boot/own.png"))),
        &mut changes,
        &state,
    )
    .unwrap();

    let page = shown(&state, &changes).unwrap();

    assert_eq!(changes.len(), 1);
    assert_eq!(page.background, "/boot/own.png");
    assert_eq!(page.background_source, PictureSource::Own);
}

#[test]
fn clearing_queues_nothing_while_only_the_theme_sets_a_picture() {
    let state = system(&with_menu_picture(), &[]);
    let mut changes = ChangeSet::default();

    set(GrubSetting::Background(None), &mut changes, &state).unwrap();

    assert!(changes.is_empty());
    assert_eq!(shown(&state, &changes).unwrap().background, THEME_PICTURE);
}

#[test]
fn clearing_an_own_picture_shows_the_one_of_the_theme_again() {
    let own = with_own_picture(&with_menu_picture());
    let state = system(&own, &[]);
    let mut changes = ChangeSet::default();
    let before = shown(&state, &changes).unwrap();

    assert_eq!(before.background, "/boot/own.png");
    assert_eq!(before.background_source, PictureSource::Own);

    set(GrubSetting::Background(None), &mut changes, &state).unwrap();

    let after = shown(&state, &changes).unwrap();

    assert_eq!(changes.len(), 1);
    assert_eq!(after.background, THEME_PICTURE);
    assert_eq!(after.background_source, PictureSource::Theme);
}

#[test]
fn without_a_theme_picture_clearing_shows_none() {
    let state = system(&with_own_picture(MOKKA), &[]);
    let mut changes = ChangeSet::default();

    set(GrubSetting::Background(None), &mut changes, &state).unwrap();

    let after = shown(&state, &changes).unwrap();

    assert_eq!(after.background, "");
    assert_eq!(after.background_source, PictureSource::Own);
}
