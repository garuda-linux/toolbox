//! The pictures rows show: the icon theme's pictures of the catalogs'
//! apps, and the Discover apps' pictures, looked up as rows show. Both
//! follow the desktop's icon theme.

use std::sync::Arc;

use crate::desktop::icon_files::AppIcons;
use crate::pictures::Pictures;

#[derive(Debug, Default)]
pub struct Artwork {
    /// `None` until the icon theme was read.
    app_icons: Option<Arc<AppIcons>>,
    /// `None` in tests, which look nothing up.
    pictures: Option<Pictures>,
}

impl Artwork {
    pub const fn new(pictures: Option<Pictures>) -> Self {
        Self {
            app_icons: None,
            pictures,
        }
    }

    pub fn app_icons(&self) -> Option<&AppIcons> {
        self.app_icons.as_deref()
    }

    pub const fn pictures(&self) -> Option<&Pictures> {
        self.pictures.as_ref()
    }

    /// The desktop's icon theme came in or changed.
    pub fn use_theme(&mut self, app_icons: Option<&Arc<AppIcons>>, theme: &str) {
        self.app_icons = app_icons.cloned();
        if let Some(pictures) = &self.pictures {
            pictures.set_theme(theme);
        }
    }

    /// A plan may have installed an app, whose picture an earlier miss
    /// would hide until the next start.
    pub fn plan_finished(&self) {
        if let Some(pictures) = &self.pictures {
            pictures.forget_misses();
        }
    }
}
