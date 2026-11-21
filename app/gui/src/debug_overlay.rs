//! F12 shows egui's debug aids, in debug builds only: the rect and size of
//! the widget under the pointer, where the next widget goes, and what the
//! pointer hits.

use egui::{Context, Id, Key, Modifiers};

/// A new look replaces the styles, so the aids are put back after it.
pub fn follow_f12(ctx: &Context, terminal_owns_keys: bool) {
    let id = Id::unique("toolbox_gui::debug_overlay");
    let pressed =
        !terminal_owns_keys && ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::F12));

    let was_shown = ctx.data(|data| data.get_temp::<bool>(id)).unwrap_or(false);
    let shown = was_shown != pressed;
    ctx.data_mut(|data| data.insert_temp(id, shown));

    if shown != showing(ctx) {
        ctx.all_styles_mut(|style| {
            style.debug.debug_on_hover = shown;
            style.debug.hover_shows_next = shown;
            style.debug.show_widget_hits = shown;
        });
    }
}

#[must_use = "it only answers a question"]
pub fn showing(ctx: &Context) -> bool {
    ctx.global_style().debug.debug_on_hover
}
