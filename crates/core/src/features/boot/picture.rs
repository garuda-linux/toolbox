//! Whose picture shows behind the boot menu. A file without
//! `GRUB_BACKGROUND` and `GRUB_MENU_PICTURE` has none, as Garuda ships it.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PictureSource {
    /// `GRUB_BACKGROUND`, or no picture at all.
    #[default]
    Own,
    /// `GRUB_MENU_PICTURE` in `/etc/default/grub`, for GRUB themes that
    /// read it.
    Theme,
}

impl PictureSource {
    pub(super) const fn of(from_theme: bool) -> Self {
        if from_theme { Self::Theme } else { Self::Own }
    }
}
