//! `/etc/grub.d/00_header` writes the background and the theme unquoted
//! into `grub.cfg`. Whether each refused character breaks the file there is
//! unverified, so the allow list keeps all of them out.

use crate::name::Reason;
use crate::{GrubPicture, GrubThemeFile};

#[test]
fn a_grub_path_holds_only_letters_digits_and_a_few_marks() {
    for path in [
        "/usr/share/backgrounds/garuda-wallpapers/Garuda_Dr460nized.png",
        "/boot/grub/themes/catppuccin-mocha/background@2x+v1,a=b:c.png",
    ] {
        GrubPicture::new(path).unwrap();
    }

    for refused in [
        '{', '}', '<', '>', '\'', '#', '?', '[', ']', '~', 'é', '\u{a0}',
    ] {
        let path = format!("/usr/share/backgrounds/a{refused}b.png");

        assert_eq!(
            GrubPicture::new(path.as_str()).unwrap_err().reason(),
            Reason::InvalidCharacter(refused),
            "{path}"
        );
    }
}

#[test]
fn a_theme_file_follows_the_same_list() {
    GrubThemeFile::new("/usr/share/grub/themes/catppuccin-mocha/theme.txt").unwrap();

    assert_eq!(
        GrubThemeFile::new("/usr/share/grub/themes/$(id)/theme.txt")
            .unwrap_err()
            .reason(),
        Reason::InvalidCharacter('$')
    );
}
