//! Which icon a frame painted, told by the file its texture was loaded
//! from, as egui names a loaded texture by its URI.

use egui::{Rect, Shape};
use egui_kittest::Harness;
use toolbox_gui_kit::icons::Icon;

/// Whether the last frame painted `icon` inside `within`.
pub fn paints_icon<State>(harness: &Harness<'_, State>, within: Rect, icon: Icon) -> bool {
    let textures = harness.ctx.tex_manager();
    let textures = textures.read();
    harness.output().shapes.iter().any(|clipped| {
        matches!(&clipped.shape, Shape::Rect(rect)
            if rect.brush.is_some()
                && within.contains_rect(rect.rect)
                && textures
                    .meta(rect.fill_texture_id())
                    .is_some_and(|meta| is_file_of(&meta.name, icon)))
    })
}

/// A file is named by the icon's theme name. A colour icon whose text parts
/// took an ink carries that ink after it as `-rrggbb`.
fn is_file_of(uri: &str, icon: Icon) -> bool {
    let name = icon.theme_name();
    let stem = uri
        .rsplit('/')
        .next()
        .unwrap_or(uri)
        .trim_end_matches(".svg");

    stem == name
        || stem
            .strip_prefix(name)
            .and_then(|ink| ink.strip_prefix('-'))
            .is_some_and(|ink| ink.len() == 6 && ink.bytes().all(|digit| digit.is_ascii_hexdigit()))
}
