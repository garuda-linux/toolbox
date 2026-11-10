//! The widgets pages use, on [`egui::Ui`] as Rerun's `UiExt` does. Every
//! one has an accessible name, so Orca and kittest find it by its text.

#[cfg(test)]
mod tests;

use egui::{Color32, Image, Response, RichText, Role, TextStyle, Ui, WidgetInfo, vec2};

use crate::icons::{self, Icon};
use crate::tokens::{self, Tokens};

/// An icon is this much taller than a line of body text, so it reads as
/// one with the text beside it.
pub(crate) const ICON_OVER_TEXT: f32 = 6.0;

pub trait UiExt {
    fn ui(&self) -> &Ui;
    fn ui_mut(&mut self) -> &mut Ui;

    fn tokens(&self) -> Tokens {
        tokens::current(self.ui().ctx())
    }

    /// An icon that says something on its own, named `alt` for screen
    /// readers.
    fn icon(&mut self, icon: Icon, alt: &str) -> Response {
        let ui = self.ui_mut();
        let tint = ui.visuals().text_color();
        let image = icon_image(ui, icon, tint).alt_text(alt);
        ui.add(image)
    }

    fn warning_label(&mut self, text: &str) -> Response {
        let ui = self.ui_mut();
        let color = tokens::current(ui.ctx()).warning;
        ui.label(RichText::new(text).color(color))
    }

    /// Pulsing dots with what is being waited for, one line tall.
    fn loading(&mut self, text: &str) -> Response {
        crate::widgets::loading_label(self.ui_mut(), text)
    }

    /// Names this ui a pane for screen readers, such as the sidebar. egui
    /// leaves every ui a nameless box, and a tree of those has no shape to
    /// find one's way by.
    fn name_pane(&self, name: &str) {
        self.ui()
            .response()
            .widget_info(|| WidgetInfo::labeled(Role::Pane, true, name));
    }
}

impl UiExt for Ui {
    fn ui(&self) -> &Ui {
        self
    }

    fn ui_mut(&mut self) -> &mut Ui {
        self
    }
}

/// `icon` from the desktop theme or bundled, sized to the body text.
pub(crate) fn icon_image(ui: &Ui, icon: Icon, tint: Color32) -> Image<'static> {
    let size = ui.text_style_height(&TextStyle::Body) + ICON_OVER_TEXT;
    let theme = icons::theme_in_use(ui.ctx());
    icon.image(theme.as_deref(), tint)
        .fit_to_exact_size(vec2(size, size))
}
