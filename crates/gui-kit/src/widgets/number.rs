//! A field for a whole number, its digits at the right as in a table.

use std::ops::RangeInclusive;

use egui::{Align2, Key, Response, TextEdit, Ui};

use super::field::{text_field, text_field_with_unit};

/// A field for `value`, `width` wide and named `name`. Only digits can be
/// typed. The number counts once Enter or leaving the field commits it,
/// held to `range`. Escape keeps the value it had.
pub fn number_field(
    ui: &mut Ui,
    value: &mut u32,
    range: RangeInclusive<u32>,
    width: f32,
    name: &str,
) -> Response {
    number(ui, (value, range), width, name, None)
}

/// A [`number_field`] with `unit`, such as "s", after the number inside
/// the field, named by the first of `(name, unit)`. The name says the
/// unit for screen readers, which do not hear the painted one.
pub fn number_field_with_unit(
    ui: &mut Ui,
    value: &mut u32,
    range: RangeInclusive<u32>,
    width: f32,
    (name, unit): (&str, &str),
) -> Response {
    number(ui, (value, range), width, name, Some(unit))
}

fn number(
    ui: &mut Ui,
    (value, range): (&mut u32, RangeInclusive<u32>),
    width: f32,
    name: &str,
    unit: Option<&str>,
) -> Response {
    let id = ui.scope_id().with(("number_field", name));
    let typing = ui.data(|data| data.get_temp::<String>(id));
    let mut text = typing.unwrap_or_else(|| value.to_string());
    let edit = TextEdit::singleline(&mut text)
        .id(id)
        .desired_width(width)
        .align(Align2::RIGHT_CENTER);

    let response = match unit {
        Some(unit) => text_field_with_unit(ui, edit, unit),
        None => text_field(ui, edit),
    };

    let label = name.to_owned();
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_label(label));

    text.retain(|typed| typed.is_ascii_digit());
    if response.has_focus() {
        ui.data_mut(|data| data.insert_temp(id, text));
        return response;
    }

    ui.data_mut(|data| data.remove_temp::<String>(id));
    let kept = ui.input(|input| input.key_pressed(Key::Escape));
    if response.lost_focus()
        && !kept
        && let Some(number) = committed(&text, &range)
    {
        *value = number;
    }

    response
}

/// The number `typed` stands for, held to `range`. `None` when nothing
/// was typed.
fn committed(typed: &str, range: &RangeInclusive<u32>) -> Option<u32> {
    if typed.is_empty() {
        return None;
    }

    // Digits past what a u32 holds are more than any range allows.
    let number = typed.parse::<u32>().unwrap_or(u32::MAX);
    Some(number.clamp(*range.start(), *range.end()))
}

#[cfg(test)]
mod tests;
