//! The desktop's look for the app: its files, its portal settings, and a
//! task that sends a new [`DesktopLook`] when either changes.

pub mod files;
pub mod follow;
pub mod font_files;
pub mod icon_files;
mod portal;

use std::sync::Arc;

use egui::FontDefinitions;
use toolbox_gui_kit::desktop::look::{Look, Sources};
use toolbox_gui_kit::fonts::{self, FontFile};
use toolbox_gui_kit::icons::ThemeIcons;

use self::icon_files::AppIcons;

/// Found off the UI thread, so the frame never scans the font folders.
#[derive(Clone, Debug, PartialEq)]
pub struct DesktopLook {
    pub look: Look,
    pub ui_fonts: Vec<FontFile>,
    pub fixed_font: Option<FontFile>,
    /// Shared, as the font is megabytes.
    pub emoji_font: Option<Arc<FontFile>>,
    pub icons: Option<Arc<ThemeIcons>>,
    pub app_icons: Option<Arc<AppIcons>>,
}

impl Default for DesktopLook {
    fn default() -> Self {
        Self {
            look: Look::resolve(&Sources::default()),
            ui_fonts: Vec::new(),
            fixed_font: None,
            emoji_font: None,
            icons: None,
            app_icons: None,
        }
    }
}

#[must_use = "the definitions do nothing until set on the context"]
pub fn font_definitions(desktop: &DesktopLook) -> FontDefinitions {
    let mut definitions = fonts::definitions(&desktop.ui_fonts, desktop.fixed_font.as_ref());
    if let Some(emoji) = &desktop.emoji_font {
        fonts::add_fallback(&mut definitions, emoji);
    }

    definitions
}
