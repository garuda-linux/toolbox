//! A field for a whole number, its digits at the right as in a table. A
//! spin button to screen readers: Up and Down step the number.

mod digits;

use std::ops::RangeInclusive;

use egui::accesskit::Role;
use egui::{Align2, EventFilter, Key, Modifiers, Response, TextEdit, Ui};

use self::digits::Digits;
use super::field::{inset_field_with_unit, text_field, text_field_with_unit};

#[derive(Clone, Copy)]
enum Look<'a> {
    Plain,
    Unit(&'a str),
    Inset(&'a str),
}

/// A field for `value`, `width` wide and named `name`.
///
/// Only digits can be typed. The number counts once Enter or leaving the
/// field commits it, held to `range`. Escape keeps the value it had. Up
/// and Down step it at once.
pub fn number_field(
    ui: &mut Ui,
    value: &mut u32,
    range: RangeInclusive<u32>,
    width: f32,
    name: &str,
) -> Response {
    number(ui, (value, range), width, name, Look::Plain)
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
    number(ui, (value, range), width, name, Look::Unit(unit))
}

/// A [`number_field_with_unit`] without its own fill, for a control
/// that paints the pill around it.
pub(super) fn inset_number_with_unit(
    ui: &mut Ui,
    value: &mut u32,
    range: RangeInclusive<u32>,
    width: f32,
    (name, unit): (&str, &str),
) -> Response {
    number(ui, (value, range), width, name, Look::Inset(unit))
}

fn number(
    ui: &mut Ui,
    (value, range): (&mut u32, RangeInclusive<u32>),
    width: f32,
    name: &str,
    look: Look<'_>,
) -> Response {
    let id = ui.scope_id().with(("number_field", name));
    let typing = ui.data(|data| data.get_temp::<String>(id));
    let mut digits = Digits(typing.unwrap_or_else(|| value.to_string()));

    // Taken before the text sees them, which would move the caret to an
    // end instead.
    if ui.memory(|memory| memory.has_focus(id))
        && let Some(up) = step_key(ui)
    {
        let from = committed(&digits.0, &range).unwrap_or(*value);
        *value = stepped(from, up, &range);
        digits.0 = value.to_string();
    }

    let edit = TextEdit::singleline(&mut digits)
        .id(id)
        .desired_width(width)
        .align(Align2::RIGHT_CENTER);

    let response = match look {
        Look::Plain => text_field(ui, edit),
        Look::Unit(unit) => text_field_with_unit(ui, edit, unit),
        Look::Inset(unit) => inset_field_with_unit(ui, edit, unit),
    };

    describe(ui, &response, name, (*value, &range));

    if response.has_focus() {
        // Up and Down stay with the field rather than move the focus.
        let filter = EventFilter {
            horizontal_arrows: true,
            vertical_arrows: true,
            ..EventFilter::default()
        };

        ui.memory_mut(|memory| memory.set_focus_lock_filter(id, filter));
        ui.data_mut(|data| data.insert_temp(id, digits.0));
        return response;
    }

    ui.data_mut(|data| data.remove_temp::<String>(id));
    let kept = ui.input(|input| input.key_pressed(Key::Escape));
    if response.lost_focus()
        && !kept
        && let Some(number) = committed(&digits.0, &range)
    {
        *value = number;
    }

    response
}

fn describe(ui: &Ui, response: &Response, name: &str, (value, range): (u32, &RangeInclusive<u32>)) {
    let label = name.to_owned();
    let (low, high) = (f64::from(*range.start()), f64::from(*range.end()));
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_role(Role::SpinButton);
        node.set_label(label);
        node.set_numeric_value(f64::from(value));
        node.set_min_numeric_value(low);
        node.set_max_numeric_value(high);
        node.set_numeric_value_step(1.0);
    });
}

/// `Some(true)` for Up, `Some(false)` for Down, taken from the input.
fn step_key(ui: &Ui) -> Option<bool> {
    ui.input_mut(|input| {
        if input.consume_key(Modifiers::NONE, Key::ArrowUp) {
            Some(true)
        } else if input.consume_key(Modifiers::NONE, Key::ArrowDown) {
            Some(false)
        } else {
            None
        }
    })
}

/// One step up or down from `from`, held to `range`.
fn stepped(from: u32, up: bool, range: &RangeInclusive<u32>) -> u32 {
    let next = if up {
        from.saturating_add(1)
    } else {
        from.saturating_sub(1)
    };

    next.clamp(*range.start(), *range.end())
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
