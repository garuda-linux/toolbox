//! The design system of the egui app, in the role Rerun's `re_ui` has
//! for its viewer: the desktop's [`Palette`] per theme (or the Catppuccin
//! fallback) becomes [`tokens::Tokens`], and [`theme::apply`] turns those
//! into egui's style. Pages use the widgets from here, never raw colours.

// wgpu's types nest deep enough that proving the live blur's state `Send`
// runs past the default limit.
#![recursion_limit = "256"]

pub mod desktop;
pub mod fonts;
pub mod frost;
pub mod icons;
pub mod jxl;
pub mod loadable;
pub mod motion;
pub mod oklch;
pub mod palette;
pub mod row_cache;
pub mod terminal_colors;
pub mod terminal_view;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
pub mod theme;
pub mod toasts;
pub mod tokens;
pub mod ui_ext;
pub mod widgets;

use egui::{Context, Theme};
pub use loadable::Loadable;
pub use palette::Palette;
pub use ui_ext::UiExt;

/// The fallback look: bundled fonts at the default size and the Catppuccin
/// palettes. Call it before the first frame, as the app and every test
/// harness do. The desktop's look replaces parts of it later.
pub fn install(ctx: &Context) {
    jxl::register();
    egui_extras::install_image_loaders(ctx);
    if !ctx.is_loader_installed(icons::ThemeLoader::ID) {
        ctx.add_image_loader(std::sync::Arc::new(icons::ThemeLoader::default()));
    }

    widgets::install_focus(ctx);
    ctx.set_fonts(fonts::definitions(&[], None));
    fonts::apply_sizes(ctx, None, None);
    theme::apply(
        ctx,
        &Palette::fallback(Theme::Dark),
        &Palette::fallback(Theme::Light),
    );
}
