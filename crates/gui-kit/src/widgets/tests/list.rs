mod rows;

use egui::accesskit::Role;
use egui::{Key, Modifiers, Rangef, ScrollArea, Ui};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::list::span::Offsets;
use super::super::{
    ListItem, RowHeights, RowList, menu_item, row_context_menu, row_menu, switch, text_block_height,
};
use crate::testing::harness;

struct Item(String);

impl ListItem for Item {
    fn key(&self) -> &str {
        &self.0
    }

    fn name(&self) -> &str {
        &self.0
    }
}

#[derive(Default)]
struct Lists {
    first: Vec<Item>,
    second: Vec<Item>,
    toggled: Vec<String>,
    acted: Vec<String>,
}

fn items(names: &[&str]) -> Vec<Item> {
    names.iter().map(|name| Item((*name).to_owned())).collect()
}

fn lists(ui: &mut Ui, state: &mut Lists) {
    let Lists {
        first,
        second,
        toggled,
        acted,
    } = state;

    ScrollArea::vertical().show(ui, |ui| {
        RowList::new("first", "First", RowHeights::Fixed(44.0)).show(ui, first, |ui, item, row| {
            ui.horizontal(|ui| {
                if switch(ui, false, &format!("Switch {}", item.0), true).clicked() || row.toggle {
                    toggled.push(item.0.clone());
                }

                row_menu(ui, row, &format!("More for {}", item.0), |ui| {
                    if menu_item(ui, "Act").clicked() {
                        acted.push(item.0.clone());
                    }
                });
            });
        });

        ui.add_space(12.0);
        let heights = RowHeights::Measured { estimate: 30.0 };
        RowList::new("second", "Second", heights).show(ui, second, |ui, item, row| {
            ui.label(format!("About {}", item.0));
            row_context_menu(ui, row, &item.0, |ui| {
                if menu_item(ui, "Act").clicked() {
                    acted.push(item.0.clone());
                }
            });
        });
    });
}

fn shown(first: &[&str], second: &[&str], height: f32) -> Harness<'static, Lists> {
    let state = Lists {
        first: items(first),
        second: items(second),
        ..Lists::default()
    };

    let mut harness = harness(400.0, height).build_ui_state(lists, state);
    harness.run();
    harness
}

fn focus_row(harness: &mut Harness<'static, Lists>, name: &str) {
    harness.get_by_role_and_label(Role::ListItem, name).focus();
    harness.run();
}

fn focused(harness: &Harness<'static, Lists>, name: &str) -> bool {
    harness
        .get_by_role_and_label(Role::ListItem, name)
        .is_focused()
}

fn press(harness: &mut Harness<'static, Lists>, key: Key) {
    harness.key_press(key);
    harness.run();
}

#[test]
fn each_row_is_a_named_list_item_in_a_named_list() {
    let harness = shown(&["a", "b"], &["c"], 400.0);

    assert!(
        harness
            .query_by_role_and_label(Role::List, "First")
            .is_some()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::ListItem, "b")
            .is_some()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::ListItem, "c")
            .is_some()
    );
}

#[test]
fn a_click_on_the_empty_part_focuses_the_row() {
    let mut harness = shown(&["a", "b"], &[], 400.0);

    harness.get_by_role_and_label(Role::ListItem, "b").click();
    harness.run();

    assert!(focused(&harness, "b"));
}

#[test]
fn the_focus_stays_with_its_item_when_the_row_above_goes() {
    let mut harness = shown(&["a", "b"], &[], 400.0);
    harness
        .get_by_role_and_label(Role::CheckBox, "Switch b")
        .focus();
    harness.run();

    harness.state_mut().first.remove(0);
    harness.run();

    assert!(
        harness
            .get_by_role_and_label(Role::CheckBox, "Switch b")
            .is_focused()
    );
}

#[test]
fn up_and_down_move_within_the_card_and_stop_at_its_ends() {
    let mut harness = shown(&["a", "b"], &["c"], 400.0);
    focus_row(&mut harness, "a");

    press(&mut harness, Key::ArrowDown);
    assert!(focused(&harness, "b"));

    press(&mut harness, Key::ArrowDown);
    assert!(focused(&harness, "b"), "the next card is Tab's");

    press(&mut harness, Key::ArrowUp);
    press(&mut harness, Key::ArrowUp);
    assert!(focused(&harness, "a"));
}

#[test]
fn tab_goes_from_a_row_to_its_controls() {
    let mut harness = shown(&["a"], &[], 400.0);
    focus_row(&mut harness, "a");

    press(&mut harness, Key::Tab);

    assert!(
        harness
            .get_by_role_and_label(Role::CheckBox, "Switch a")
            .is_focused()
    );
}

#[test]
fn space_on_a_focused_row_flips_its_switch() {
    let mut harness = shown(&["a", "b"], &[], 400.0);
    focus_row(&mut harness, "b");

    press(&mut harness, Key::Space);

    assert_eq!(harness.state().toggled, ["b"]);
}

#[test]
fn the_menu_key_opens_the_row_menu_and_escape_closes_it() {
    let mut harness = shown(&["a"], &[], 400.0);
    focus_row(&mut harness, "a");

    press(&mut harness, Key::Menu);
    assert!(
        harness
            .query_by_role_and_label(Role::MenuItem, "Act")
            .is_some()
    );

    press(&mut harness, Key::Escape);
    assert!(
        harness
            .query_by_role_and_label(Role::MenuItem, "Act")
            .is_none()
    );
}

#[test]
fn shift_f10_opens_the_menu_of_a_row_without_a_button() {
    let mut harness = shown(&[], &["c"], 400.0);
    focus_row(&mut harness, "c");

    harness.key_press_modifiers(Modifiers::SHIFT, Key::F10);
    harness.run();
    harness.get_by_role_and_label(Role::MenuItem, "Act").click();
    harness.run();

    assert_eq!(harness.state().acted, ["c"]);
}

#[test]
fn a_right_click_opens_the_same_menu_as_the_button() {
    let mut harness = shown(&["a"], &[], 400.0);
    let row = harness.get_by_role_and_label(Role::ListItem, "a");
    let at = row.rect().center() + egui::vec2(120.0, 0.0);
    harness.hover_at(at);
    harness.event(egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Secondary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    harness.event(egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Secondary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    harness.run();

    harness.get_by_role_and_label(Role::MenuItem, "Act").click();
    harness.run();

    assert_eq!(harness.state().acted, ["a"]);
}

fn many() -> Vec<String> {
    (0..300).map(|index| format!("row {index:03}")).collect()
}

#[test]
fn only_the_rows_near_the_view_are_laid_out() {
    let names = many();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let harness = shown(&names, &[], 300.0);

    let laid_out = harness.query_all_by_role(Role::ListItem).count();

    assert!(laid_out < 12, "{laid_out} rows laid out");
    assert!(
        harness
            .query_by_role_and_label(Role::ListItem, "row 299")
            .is_none()
    );
}

#[test]
fn the_focused_row_stays_laid_out_and_comes_into_view() {
    let names = many();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let mut harness = shown(&names, &[], 300.0);
    focus_row(&mut harness, "row 000");

    for _ in 0..20_u8 {
        press(&mut harness, Key::ArrowDown);
    }

    let row = harness.get_by_role_and_label(Role::ListItem, "row 020");
    assert!(row.is_focused());
    assert!(row.rect().bottom() <= 300.0, "{:?}", row.rect());
}

#[test]
fn spans_keep_one_row_beyond_each_side_and_the_kept_row() {
    let offsets = Offsets::Fixed {
        height: 10.0,
        count: 100,
    };

    assert_eq!(
        offsets.rows_to_draw(Rangef::new(25.0, 45.0), Some(90)),
        [1, 2, 3, 4, 5, 90]
    );
    assert_eq!(offsets.rows_to_draw(Rangef::new(-50.0, 5.0), None), [0, 1]);
}

#[test]
fn measured_spans_add_the_heights_up() {
    let offsets = Offsets::measured([10.0, 30.0, 10.0]);

    assert!((offsets.top(2) - 40.0).abs() < f32::EPSILON);
    assert!((offsets.total() - 50.0).abs() < f32::EPSILON);
    assert_eq!(offsets.index_at(15.0), 1);
}

#[test]
fn a_larger_body_font_makes_taller_rows() {
    let mut heights = Vec::new();
    for size in [13.0, 20.0] {
        let mut harness = harness(200.0, 100.0).build_ui_state(
            move |ui, height: &mut f32| {
                ui.style_mut()
                    .text_styles
                    .insert(egui::TextStyle::Body, egui::FontId::proportional(size));
                *height = text_block_height(ui, 2);
            },
            0.0,
        );

        harness.run();
        heights.push(*harness.state());
    }

    assert!(heights[1] > heights[0] + 10.0, "{heights:?}");
}

#[test]
fn a_list_out_of_view_lays_out_only_the_row_next_to_the_view() {
    let offsets = Offsets::Fixed {
        height: 10.0,
        count: 100,
    };

    assert_eq!(offsets.rows_to_draw(Rangef::new(-80.0, -20.0), None), [0]);
    assert_eq!(
        offsets.rows_to_draw(Rangef::new(1200.0, 1300.0), None),
        [99]
    );
}

#[test]
fn a_row_keeps_its_hover_under_its_own_button_but_not_under_a_drag() {
    use super::super::card::shows_hover;

    assert!(shows_hover(true, true, false));
    assert!(shows_hover(false, true, false), "its own button is pressed");
    assert!(!shows_hover(false, true, true), "something else is dragged");
    assert!(!shows_hover(false, false, false));
}

#[test]
fn a_revealed_row_far_below_takes_the_focus_and_scrolls_into_view() {
    let rows: Vec<Item> = (0..60_u8)
        .map(|row| Item(format!("row {row:02}")))
        .collect();

    let mut harness = harness(300.0, 200.0).build_ui_state(
        move |ui, reveal: &mut Option<&'static str>| {
            ScrollArea::vertical().show(ui, |ui| {
                RowList::new("rows", "Rows", RowHeights::Fixed(56.0))
                    .reveal(reveal.take())
                    .show(ui, &rows, |ui, item, _| drop(ui.label(&item.0)));
            });
        },
        Some("row 50"),
    );

    harness.run();

    let row = harness.get_by_role_and_label(Role::ListItem, "row 50");
    assert!(row.is_focused());
    assert!(row.rect().bottom() <= 200.0, "{:?}", row.rect());
}

#[test]
fn a_click_on_a_row_flips_its_switch_once() {
    let mut harness = shown(&["a", "b"], &[], 400.0);

    harness.get_by_role_and_label(Role::ListItem, "b").click();
    harness.run();

    assert_eq!(harness.state().toggled, ["b"]);
}
