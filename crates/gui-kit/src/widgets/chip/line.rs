//! The chip groups: chips that wrap onto as many lines as their width needs.

use egui::{AsIdSalt, Role, Ui, UiBuilder, vec2};

use super::super::Choice;
use super::super::roving::{Axes, Roving, Setup};
use super::chip_in;
use crate::tokens;

/// Returns the option clicked this frame. `salt` keeps the group's Tab stop
/// its own when the page around it changes.
pub fn chips<T>(ui: &mut Ui, salt: impl AsIdSalt, options: &[(T, &str)], current: T) -> Choice<T>
where
    T: PartialEq + Copy,
{
    counted_chips(ui, salt, &uncounted(options), current)
}

/// [`chips`] in a group that `name` names, for a page with more than one
/// row of chips: each row's "All" then tells which row it belongs to.
///
/// The name is also the id salt, so it must be unique among the rows.
pub fn named_chips<T>(ui: &mut Ui, name: &str, options: &[(T, &str)], current: T) -> Choice<T>
where
    T: PartialEq + Copy,
{
    let builder = UiBuilder::new()
        .id_salt(name)
        .accessibility_role(Role::Group)
        .accessibility_label(name);

    ui.scope_builder(builder, |ui| {
        chip_row(ui, &uncounted(options), |value| value == current)
    })
    .inner
}

pub fn counted_chips<T>(
    ui: &mut Ui,
    salt: impl AsIdSalt,
    options: &[(T, &str, Option<&str>)],
    current: T,
) -> Choice<T>
where
    T: PartialEq + Copy,
{
    ui.scope_builder(UiBuilder::new().id_salt(salt), |ui| {
        chip_row(ui, options, |value| value == current)
    })
    .inner
}

/// Returns the option clicked this frame.
pub fn toggle_chips<T>(
    ui: &mut Ui,
    salt: impl AsIdSalt,
    options: &[(T, &str)],
    on: impl Fn(T) -> bool,
) -> Choice<T>
where
    T: Copy,
{
    ui.scope_builder(UiBuilder::new().id_salt(salt), |ui| {
        chip_row(ui, &uncounted(options), on)
    })
    .inner
}

fn uncounted<'a, T>(options: &[(T, &'a str)]) -> Vec<(T, &'a str, Option<&'a str>)>
where
    T: Copy,
{
    options
        .iter()
        .map(|(value, label)| (*value, *label, None))
        .collect()
}

/// Chips that wrap onto as many lines as their width needs. The group takes
/// one Tab stop, on the first selected chip, and the arrow keys, Home and
/// End move along the chips in reading order.
fn chip_row<T>(
    ui: &mut Ui,
    options: &[(T, &str, Option<&str>)],
    selected: impl Fn(T) -> bool,
) -> Choice<T>
where
    T: Copy,
{
    let tokens = tokens::current(ui.ctx());
    let gap = tokens.sizes.control_gap;

    // The scope the caller gives the group keeps its ids apart, so the
    // group's own id is an automatic one inside it.
    let group = ui.next_auto_id();
    let item = |index: usize| group.with(index);
    let rest = options.iter().position(|(value, _, _)| selected(*value));
    let roving = Roving::read(
        ui,
        group,
        options.len(),
        item,
        Setup {
            rest,
            axes: Axes::Both,
        },
    );

    let row = ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(gap, gap);
        let mut clicked = None;
        for (index, (value, label, count)) in options.iter().enumerate() {
            let id = item(index);
            let stop = roving.is_stop(index);
            let chip = chip_in(ui, label, *count, selected(*value), Some((id, stop)));
            roving.apply(ui, index, id, &chip);
            if chip.clicked() {
                clicked = Some(*value);
            }
        }

        clicked
    });

    Choice {
        response: row.response,
        value: row.inner,
    }
}
