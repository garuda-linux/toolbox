use egui::{
    Align, Atom, Context, Frame, Id, IdSalt, Layout, Margin, Rect, Response, Role, Sense, Shape,
    TextEdit, TextStyle, Ui, UiBuilder, WidgetInfo, pos2, vec2,
};

use super::buttons::{SMALL_BUTTON, paint_small_button};
use super::field::{PADDING_X, paint_field};
use super::rich;
use crate::fonts::{BODY, Weight};
use crate::theme::{PILL_HEIGHT, PILL_RADIUS};
use crate::ui_ext::icon_image;
use crate::{icons, tokens};

#[derive(Clone, Copy, Debug)]
pub struct SearchTexts<'a> {
    /// What screen readers call the field, on the search glyph that
    /// labels it.
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

/// Its clear button empties it and puts the focus back.
pub fn search_field(
    ui: &mut Ui,
    text: &mut String,
    texts: SearchTexts<'_>,
    width: f32,
) -> Response {
    let tokens = tokens::current(ui.ctx());
    let id = ui.make_persistent_id(("search", texts.name));
    if take_focus_request(ui) {
        ui.memory_mut(|memory| memory.request_focus(id));
    }

    let focused = ui.memory(|memory| memory.has_focus(id));

    let (rect, _) = ui.allocate_exact_size(vec2(width, PILL_HEIGHT), Sense::hover());
    let slot = ui.painter().add(Shape::Noop);
    paint_field(ui, slot, rect, PILL_RADIUS, focused);

    let clear_room = if text.is_empty() { 0.0 } else { SMALL_BUTTON };
    let inner = Rect::from_min_max(
        pos2(rect.left() + f32::from(PADDING_X), rect.top()),
        pos2(
            rect.right() - f32::from(PADDING_X).max(clear_room + tokens.sizes.tight_gap),
            rect.bottom(),
        ),
    );

    let layout = Layout::left_to_right(Align::Center);

    let response = ui
        .scope_builder(UiBuilder::new().max_rect(inner).layout(layout), |ui| {
            ui.spacing_mut().interact_size.y = 0.0;
            let glyph = icon_image(ui, icons::SYSTEM_SEARCH, tokens.dim).alt_text(texts.name);
            let label = ui.add(glyph);
            let hint = rich(ui, texts.hint, Weight::Regular, BODY, tokens.secondary);

            // egui paints the caret of an empty field in the hint's first
            // atom. A caret wide one keeps it off the hint's first letter.
            let caret = Atom::custom(
                IdSalt::new("search-caret"),
                vec2(
                    ui.visuals().text_cursor.stroke.width,
                    ui.text_style_height(&TextStyle::Body),
                ),
            );

            ui.add(
                TextEdit::singleline(text)
                    .id(id)
                    .frame(Frame::NONE)
                    .margin(Margin::ZERO)
                    .align(egui::Align2::LEFT_CENTER)
                    .hint_text((caret, hint))
                    .text_color(tokens.text)
                    .desired_width(f32::INFINITY),
            )
            .labelled_by(label.id)
        })
        .inner;

    if !text.is_empty() && clear_button(ui, rect, id, texts.clear) {
        text.clear();
        ui.memory_mut(|memory| memory.request_focus(id));
    }

    response
}

fn take_focus_request(ui: &Ui) -> bool {
    let pass = ui.ctx().cumulative_pass_nr();
    ui.data_mut(|data| data.remove_temp::<u64>(focus_request_id()))
        .is_some_and(|asked| pass.saturating_sub(asked) <= 1)
}

fn clear_button(ui: &Ui, field: Rect, field_id: Id, alt: &str) -> bool {
    let rect = Rect::from_center_size(
        pos2(field.right() - PILL_HEIGHT / 2.0, field.center().y),
        vec2(SMALL_BUTTON, SMALL_BUTTON),
    );

    let response = ui.interact(rect, field_id.with("clear"), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), alt));
    let pressed = tokens::current(ui.ctx()).pressed;
    paint_small_button(ui, rect, &response, icons::WINDOW_CLOSE, pressed);
    response.on_hover_text(alt).clicked()
}

fn focus_request_id() -> Id {
    Id::unique("toolbox_gui_kit::search::focus_request")
}
