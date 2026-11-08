use super::super::{GrubSetting, GrubState};
use super::{INSTALLED, LIMA, changed_lines};
use crate::GrubPicture;

const THEME_PICTURE: &str = "/usr/share/grub/themes/catppuccin-mocha/background.png";

fn picture(path: &str) -> GrubPicture {
    GrubPicture::new(path).unwrap()
}

/// Edit: a `GRUB_MENU_PICTURE` line appended to the real install, as no
/// captured `/etc/default/grub` sets it.
fn with_menu_picture() -> String {
    format!("{INSTALLED}export GRUB_MENU_PICTURE=\"{THEME_PICTURE}\"\n")
}

#[test]
fn the_captured_files_set_neither_picture_variable() {
    for config in [LIMA, INSTALLED] {
        let state = GrubState::read(config);

        assert_eq!(state.background, "");
        assert_eq!(state.menu_picture, "");
        assert_eq!(state.shown_background(), "");
        assert!(!state.background_is_from_theme());
    }
}

#[test]
fn a_menu_picture_line_shows_the_picture_of_the_theme() {
    let state = GrubState::read(&with_menu_picture());

    assert_eq!(state.background, "");
    assert_eq!(state.menu_picture, THEME_PICTURE);
    assert_eq!(state.shown_background(), THEME_PICTURE);
    assert!(state.background_is_from_theme());
}

#[test]
fn the_background_variable_wins_over_the_theme_and_an_empty_one_does_not() {
    let themed = with_menu_picture();
    let own = format!("{themed}GRUB_BACKGROUND=\"/boot/own.png\"\n");
    let empty = format!("{themed}GRUB_BACKGROUND=\"\"\n");

    assert_eq!(GrubState::read(&own).shown_background(), "/boot/own.png");
    assert!(!GrubState::read(&own).background_is_from_theme());
    assert_eq!(GrubState::read(&empty).shown_background(), THEME_PICTURE);
    assert!(GrubState::read(&empty).background_is_from_theme());
}

#[test]
fn a_new_picture_is_written_to_the_background_variable_and_the_theme_line_stays() {
    let themed = with_menu_picture();
    let after = GrubSetting::Background(Some(picture("/boot/own.png"))).apply(&themed);

    assert_eq!(
        changed_lines(&themed, &after),
        ["GRUB_BACKGROUND=\"/boot/own.png\""]
    );
    assert!(after.contains(&format!("export GRUB_MENU_PICTURE=\"{THEME_PICTURE}\"")));

    let state = GrubState::read(&after);

    assert_eq!(state.shown_background(), "/boot/own.png");
    assert_eq!(state.menu_picture, THEME_PICTURE);
}

#[test]
fn taking_the_own_picture_away_brings_the_one_of_the_theme_back() {
    let own = GrubSetting::Background(Some(picture("/boot/own.png"))).apply(&with_menu_picture());
    let after = GrubSetting::Background(None).apply(&own);
    let state = GrubState::read(&after);

    assert_eq!(state.background, "");
    assert_eq!(state.shown_background(), THEME_PICTURE);
    assert!(after.contains("export GRUB_MENU_PICTURE="));
}

/// A wish is about `GRUB_BACKGROUND` alone, so the installed file already
/// has "no own picture" and choosing the theme's picture is a change.
#[test]
fn a_wish_is_met_by_the_background_variable_alone() {
    let themed = with_menu_picture();
    let stock = GrubState::read(&themed);

    assert!(GrubSetting::Background(None).is_met_by(&stock));
    assert!(!GrubSetting::Background(Some(picture(THEME_PICTURE))).is_met_by(&stock));
    assert_eq!(GrubSetting::Background(None).apply(&themed), themed);
    assert_eq!(GrubSetting::Background(None).apply(INSTALLED), INSTALLED);
}
