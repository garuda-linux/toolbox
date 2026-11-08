use super::super::{GrubSetting, GrubState};
use super::{ARCH, GARUDA, changed_lines};
use crate::GrubPicture;

const THEME_PICTURE: &str = "/usr/share/grub/themes/garuda-dr460nized/background.png";

fn picture(path: &str) -> GrubPicture {
    GrubPicture::new(path).unwrap()
}

/// Garuda's stock file sets only `GRUB_MENU_PICTURE`, so it must not read
/// as no picture.
#[test]
fn the_garuda_file_shows_the_picture_of_its_theme() {
    let state = GrubState::read(GARUDA);

    assert_eq!(state.background, "");
    assert_eq!(state.menu_picture, THEME_PICTURE);
    assert_eq!(state.shown_background(), THEME_PICTURE);
    assert!(state.background_is_from_theme());
}

#[test]
fn a_file_without_either_variable_has_no_picture() {
    let state = GrubState::read(ARCH);

    assert_eq!(state.shown_background(), "");
    assert!(!state.background_is_from_theme());
}

#[test]
fn the_background_variable_wins_over_the_theme_and_an_empty_one_does_not() {
    let own = format!("{GARUDA}GRUB_BACKGROUND=\"/boot/own.png\"\n");
    let empty = format!("{GARUDA}GRUB_BACKGROUND=\"\"\n");

    assert_eq!(GrubState::read(&own).shown_background(), "/boot/own.png");
    assert!(!GrubState::read(&own).background_is_from_theme());
    assert_eq!(GrubState::read(&empty).shown_background(), THEME_PICTURE);
    assert!(GrubState::read(&empty).background_is_from_theme());
}

#[test]
fn a_new_picture_is_written_to_the_background_variable_and_the_theme_line_stays() {
    let after = GrubSetting::Background(Some(picture("/boot/own.png"))).apply(GARUDA);

    assert_eq!(
        changed_lines(GARUDA, &after),
        ["GRUB_BACKGROUND=\"/boot/own.png\""]
    );
    assert!(after.contains(&format!("export GRUB_MENU_PICTURE=\"{THEME_PICTURE}\"")));

    let state = GrubState::read(&after);

    assert_eq!(state.shown_background(), "/boot/own.png");
    assert_eq!(state.menu_picture, THEME_PICTURE);
}

#[test]
fn taking_the_own_picture_away_brings_the_one_of_the_theme_back() {
    let own = GrubSetting::Background(Some(picture("/boot/own.png"))).apply(GARUDA);
    let after = GrubSetting::Background(None).apply(&own);
    let state = GrubState::read(&after);

    assert_eq!(state.background, "");
    assert_eq!(state.shown_background(), THEME_PICTURE);
    assert!(after.contains("export GRUB_MENU_PICTURE="));
}

/// A wish is about `GRUB_BACKGROUND` alone, so the stock file already
/// has "no own picture" and choosing the theme's picture is a change.
#[test]
fn a_wish_is_met_by_the_background_variable_alone() {
    let stock = GrubState::read(GARUDA);

    assert!(GrubSetting::Background(None).is_met_by(&stock));
    assert!(!GrubSetting::Background(Some(picture(THEME_PICTURE))).is_met_by(&stock));
    assert_eq!(GrubSetting::Background(None).apply(GARUDA), GARUDA);
}
