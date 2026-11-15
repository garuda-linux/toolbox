use egui::{
    Align2, AsIdSalt, Atom, Context, EventFilter, Id, IdSalt, Key, Modifiers, Response, TextEdit,
    TextStyle, Ui, vec2,
};

use super::field::{FieldButton, Parts, field};
use super::rich;
use crate::fonts::{BODY, Weight};
use crate::{icons, tokens};

#[derive(Clone, Copy, Debug)]
pub struct SearchTexts<'a> {
    /// What screen readers call the field.
    pub name: &'a str,
    pub hint: &'a str,
    pub clear: &'a str,
}

/// A page without one lets the wish lapse, so a later page does not grab
/// the focus.
pub fn request_search_focus(ctx: &Context) {
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|data| data.insert_temp(focus_request_id(), pass));
}

/// A [`super::text_field`] `width` wide with the search glyph before the
/// text. Its clear button empties it and puts the focus back. Escape
/// empties a field with text and leaves an empty one.
pub fn search_field(
    ui: &mut Ui,
    salt: impl AsIdSalt,
    text: &mut String,
    texts: SearchTexts<'_>,
    width: f32,
) -> Response {
    let tokens = tokens::current(ui.ctx());
    let id = ui.make_persistent_id(("search", salt));
    if take_focus_request(ui) {
        ui.memory_mut(|memory| memory.request_focus(id));
    }

    // Taken before the text sees it, so the field keeps the focus. Once
    // empty, Escape leaves the field as before.
    let escaped = !text.is_empty()
        && ui.memory(|memory| memory.has_focus(id))
        && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape));

    if escaped {
        text.clear();
    }

    let hint = rich(ui, texts.hint, Weight::Regular, BODY, tokens.secondary);

    // egui paints the caret of an empty field in the hint's first atom. A
    // caret wide one keeps it off the hint's first letter.
    let caret = Atom::custom(
        IdSalt::new("search-caret"),
        vec2(
            ui.visuals().text_cursor.stroke.width,
            ui.text_style_height(&TextStyle::Body),
        ),
    );

    let hint_gap = Atom {
        size: Some(vec2(ui.spacing().icon_spacing, 0.0)),
        ..Atom::default()
    };

    let clear = FieldButton {
        icon: icons::WINDOW_CLOSE,
        name: texts.clear,
    };

    let parts = Parts {
        glyph: Some(icons::SYSTEM_SEARCH),
        button: (!text.is_empty()).then_some(clear),
        pill: true,
        ..Parts::default()
    };

    let edit = TextEdit::singleline(text)
        .id(id)
        .align(Align2::LEFT_CENTER)
        .hint_text((caret, hint_gap, hint))
        .text_color(tokens.text)
        .desired_width(width);

    let (mut response, cleared) = field(ui, edit, parts);
    if escaped {
        response.mark_changed();
    }

    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_label(texts.name));

    if cleared {
        text.clear();
        ui.memory_mut(|memory| memory.request_focus(id));
    }

    // egui lets go of the focus on Escape before any widget sees it. A
    // field with text keeps Escape for itself, to empty it.
    if response.has_focus() {
        let filter = EventFilter {
            horizontal_arrows: true,
            escape: !text.is_empty(),
            ..EventFilter::default()
        };

        ui.memory_mut(|memory| memory.set_focus_lock_filter(id, filter));
    }

    response
}

fn take_focus_request(ui: &Ui) -> bool {
    let pass = ui.ctx().cumulative_pass_nr();
    ui.data_mut(|data| data.remove_temp::<u64>(focus_request_id()))
        .is_some_and(|asked| pass.saturating_sub(asked) <= 1)
}

fn focus_request_id() -> Id {
    Id::unique("toolbox_gui_kit::search::focus_request")
}
