//! The name rules of the Boot options page. Every value lands inside a
//! double quoted string of `/etc/default/grub`, which a shell reads as
//! root, so none may hold a character that ends the quotes or starts an
//! expansion there. The theme name is an argument of a program.

#[cfg(test)]
mod tests;

use super::rules::{check_characters, check_shape, check_word};
use super::{Name, NameKind, Reason, Rule};
use crate::component::has_plain_components;

const SHELL_ACTIVE: &str = "\"\\$`";

/// `/etc/grub.d/00_header` expands the theme and the background unquoted,
/// so one of these in a path would split it or add a command to
/// `grub.cfg`.
const PATH_UNQUOTED: &str = " \t;|&()*";

/// The other characters real kernel parameters use beside letters and
/// digits (`root=UUID=…`, `video=1920x1080@60`, `acpi_osi=!`, `ip=[::1]`).
const PARAMETER_MARKS: &str = "_-.,:=/@+%!~^[]";

const PICTURE_TYPES: [&str; 4] = [".png", ".jpg", ".jpeg", ".tga"];

/// A GRUB theme is a folder, chosen by its description file.
const THEME_TYPES: [&str; 1] = [".txt"];

const MAX_PATH_CHARS: usize = 1024;
const MAX_KERNEL_PARAMETER_CHARS: usize = 255;
const MAX_PLYMOUTH_THEME_CHARS: usize = 64;
const MAX_MENU_ENTRY_CHARS: usize = 512;

pub type KernelParameter = Name<KernelParameterRule>;
/// A folder name below `/usr/share/plymouth/themes`.
pub type PlymouthTheme = Name<PlymouthThemeRule>;
/// What `GRUB_DEFAULT` names: a menu entry's id or title, with `>`
/// between a submenu and its entry.
pub type MenuEntryId = Name<MenuEntryRule>;
/// The picture behind the GRUB menu, as an absolute path.
pub type GrubPicture = Name<GrubPictureRule>;
/// The `theme.txt` of a GRUB theme, as an absolute path.
pub type GrubThemeFile = Name<GrubThemeRule>;

pub struct KernelParameterRule;
pub struct PlymouthThemeRule;
pub struct MenuEntryRule;
pub struct GrubPictureRule;
pub struct GrubThemeRule;

impl Rule for KernelParameterRule {
    const KIND: NameKind = NameKind::KernelParameter;

    fn check(value: &str) -> Result<(), Reason> {
        let allowed = |character: char| {
            character.is_ascii_alphanumeric() || PARAMETER_MARKS.contains(character)
        };

        check_shape(value, MAX_KERNEL_PARAMETER_CHARS, allowed)?;
        check_characters(value, allowed)
    }
}

/// `plymouth-set-default-theme` knows no `--`, so the first character
/// keeps the name from being an option.
impl Rule for PlymouthThemeRule {
    const KIND: NameKind = NameKind::PlymouthTheme;

    fn check(value: &str) -> Result<(), Reason> {
        check_shape(value, MAX_PLYMOUTH_THEME_CHARS, |first| {
            first.is_ascii_alphanumeric()
        })?;
        check_word(value)
    }
}

/// Titles hold blanks (grub-btrfs even starts with them), commas and brackets,
/// so this is a deny list: no control character and nothing a shell acts on
/// inside double quotes. No `-` at the start, so an id never reads as an
/// option.
impl Rule for MenuEntryRule {
    const KIND: NameKind = NameKind::MenuEntry;

    fn check(value: &str) -> Result<(), Reason> {
        check_shape(value, MAX_MENU_ENTRY_CHARS, |first| first != '-')?;
        check_quotable(value)
    }
}

impl Rule for GrubPictureRule {
    const KIND: NameKind = NameKind::GrubPicture;

    fn check(value: &str) -> Result<(), Reason> {
        check_file(value, &PICTURE_TYPES)
    }
}

impl Rule for GrubThemeRule {
    const KIND: NameKind = NameKind::GrubTheme;

    fn check(value: &str) -> Result<(), Reason> {
        check_file(value, &THEME_TYPES)
    }
}

fn check_quotable(value: &str) -> Result<(), Reason> {
    check_characters(value, |character| {
        !character.is_control() && !SHELL_ACTIVE.contains(character)
    })
}

/// A path that also survives the unquoted expansion in `/etc/grub.d`.
fn check_path(value: &str) -> Result<(), Reason> {
    check_quotable(value)?;
    check_characters(value, |character| !PATH_UNQUOTED.contains(character))
}

/// An absolute path without `.`, `..` or empty components whose name ends
/// in one of `types`, whatever the case.
fn check_file(value: &str, types: &[&str]) -> Result<(), Reason> {
    check_shape(value, MAX_PATH_CHARS, |first| first == '/')?;
    check_path(value)?;

    let plain = value.strip_prefix('/').is_some_and(has_plain_components);

    if !plain {
        return Err(Reason::InvalidComponent);
    }

    let name = value.to_lowercase();

    if types.iter().any(|suffix| name.ends_with(suffix)) {
        Ok(())
    } else {
        Err(Reason::WrongFileType)
    }
}
