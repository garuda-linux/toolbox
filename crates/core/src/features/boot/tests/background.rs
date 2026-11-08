use toolbox_plan::{GrubPicture, GrubSetting};

use super::super::{PictureSource, set, shown};
use super::system;
use crate::ChangeSet;

/// What Garuda's own `/etc/default/grub` says about pictures: only the
/// variable its theme reads.
const GARUDA: &str = "GRUB_DEFAULT=0\n#GRUB_BACKGROUND=\"/path/to/wallpaper\"\nexport GRUB_MENU_PICTURE=\"/usr/share/grub/themes/garuda/background.png\"\n";
const THEME_PICTURE: &str = "/usr/share/grub/themes/garuda/background.png";

fn picture(path: &str) -> GrubPicture {
    GrubPicture::new(path).unwrap()
}

#[test]
fn a_stock_garuda_file_shows_the_picture_of_its_theme_and_not_none() {
    let state = system(GARUDA, &[]);
    let page = shown(&state, &ChangeSet::default()).unwrap();

    assert_eq!(page.background, THEME_PICTURE);
    assert_eq!(page.background_source, PictureSource::Theme);
}

#[test]
fn an_own_picture_queues_and_replaces_the_one_of_the_theme_on_the_page() {
    let state = system(GARUDA, &[]);
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
    let state = system(GARUDA, &[]);
    let mut changes = ChangeSet::default();

    set(GrubSetting::Background(None), &mut changes, &state).unwrap();

    assert!(changes.is_empty());
    assert_eq!(shown(&state, &changes).unwrap().background, THEME_PICTURE);
}

#[test]
fn clearing_an_own_picture_shows_the_one_of_the_theme_again() {
    let own = format!("{GARUDA}GRUB_BACKGROUND=\"/boot/own.png\"\n");
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
    let state = system("GRUB_BACKGROUND=\"/boot/own.png\"\n", &[]);
    let mut changes = ChangeSet::default();

    set(GrubSetting::Background(None), &mut changes, &state).unwrap();

    let after = shown(&state, &changes).unwrap();

    assert_eq!(after.background, "");
    assert_eq!(after.background_source, PictureSource::Own);
}
