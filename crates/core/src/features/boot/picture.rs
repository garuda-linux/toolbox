//! Whose picture shows behind the boot menu. Garuda's stock file sets
//! none of its own and leaves it to the GRUB theme.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PictureSource {
    /// `GRUB_BACKGROUND`, or no picture at all.
    #[default]
    Own,
    /// `GRUB_MENU_PICTURE`, which the GRUB theme of Garuda reads.
    Theme,
}

impl PictureSource {
    pub(super) const fn of(from_theme: bool) -> Self {
        if from_theme { Self::Theme } else { Self::Own }
    }
}
