use egui::Shape;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::{menu_button, menu_item, menu_separator};
use crate::testing::harness;
use crate::tokens;

fn copy_menu(width: f32) -> Harness<'static> {
    let mut harness = harness(width, 300.0).build_ui(|ui| {
        menu_button(ui, "More actions", |ui| {
            drop(menu_item(ui, "Copy"));
        });
    });

    harness.run();
    harness.get_by_label("More actions").click_accesskit();
    harness.run();
    harness
}

#[test]
fn a_menu_item_picked_without_the_pointer_closes_the_menu() {
    let mut harness = harness(300.0, 300.0).build_ui_state(
        |ui, picked: &mut bool| {
            menu_button(ui, "More actions", |ui| {
                *picked |= menu_item(ui, "Copy").clicked();
            });
        },
        false,
    );

    harness.run();
    harness.get_by_label("More actions").click_accesskit();
    harness.run();
    harness.get_by_label("Copy").click_accesskit();
    harness.run();

    assert!(*harness.state());
    assert!(
        harness.query_by_label("Copy").is_none(),
        "the menu stayed open"
    );
}

/// The entry goes with the menu, so a dialog it opens would have no
/// opener to give the focus back to.
#[test]
fn a_picked_menu_item_hands_the_focus_to_the_menu_button() {
    let mut harness = copy_menu(300.0);
    harness.get_by_label("Copy").click_accesskit();
    harness.run();

    assert!(harness.get_by_label("More actions").is_focused());
}

/// Darkly's item fill is the view's hover colour, inset by one point, and
/// the rect around it is the item's own.
#[test]
fn a_hovered_menu_item_fills_its_rect_inset_by_one_point_in_the_hover_colour() {
    let mut harness = copy_menu(300.0);
    harness.get_by_label("Copy").hover();
    harness.run();

    let item = harness.get_by_label("Copy").rect();
    let hover = tokens::current(&harness.ctx).decoration_hover;
    let inset = item.shrink(1.0);
    let filled = harness.output().shapes.iter().any(|clipped| {
        matches!(&clipped.shape, Shape::Rect(rect)
            if rect.fill == hover
                && (rect.rect.min - inset.min).length() < 0.5
                && (rect.rect.max - inset.max).length() < 0.5)
    });

    assert!(filled, "no hover fill inset by 1 point in {item:?}");
}

/// A menu item is as tall as an entry of a Plasma QML list (`ItemDelegate`),
/// 30 points, so menus and dropdowns read like the rest of the desktop.
#[test]
fn a_menu_item_is_thirty_points_tall() {
    let harness = copy_menu(300.0);

    let item = harness.get_by_label("Copy").rect();

    assert!((item.height() - 30.0).abs() < 0.5, "{item:?}");
}

/// A menu is as wide as its widest entry from its second frame on, not the
/// fixed width of a dropdown.
#[test]
fn a_menu_is_as_wide_as_its_widest_entry() {
    let mut harness = copy_menu(600.0);
    harness.run();

    let item = harness.get_by_label("Copy").rect();
    assert!(item.width() < 120.0, "the menu is {} wide", item.width());
}

#[test]
fn a_menu_separator_is_a_rule_inset_ten_points_in_the_separator_colour() {
    let mut harness = harness(300.0, 300.0).build_ui(|ui| {
        menu_button(ui, "More actions", |ui| {
            drop(menu_item(ui, "Copy"));
            menu_separator(ui);
            drop(menu_item(ui, "Paste"));
        });
    });

    harness.run();
    harness.get_by_label("More actions").click_accesskit();
    harness.run();

    let copy = harness.get_by_label("Copy").rect();
    let paste = harness.get_by_label("Paste").rect();
    let separator = tokens::current(&harness.ctx).separator;
    let rule = harness.output().shapes.iter().find_map(|clipped| {
        let Shape::LineSegment { points, stroke } = &clipped.shape else {
            return None;
        };

        let between = points[0].y > copy.bottom() && points[0].y < paste.top();
        let level = (points[0].y - points[1].y).abs() < 0.01;
        (stroke.color == separator && level && between).then_some(*points)
    });

    let [left, right] = rule.expect("the separator's rule");
    assert!(
        left.y > copy.bottom() && left.y < paste.top(),
        "rule at {left:?} between entries"
    );
    assert!(
        (left.x - (copy.left() + 10.0)).abs() < 0.5,
        "{left:?} in {copy:?}"
    );
    assert!(
        (right.x - (copy.right() - 10.0)).abs() < 0.5,
        "{right:?} in {copy:?}"
    );
}

/// Darkly's strong focus draws a selected item's text in the highlighted
/// text colour over the fill (darklystyle.cpp `drawMenuItemControl`).
#[test]
fn a_hovered_menu_item_draws_its_text_in_the_highlighted_text_colour() {
    let mut harness = copy_menu(300.0);
    harness.get_by_label("Copy").hover();
    harness.run();

    let text = crate::testing::painted_text(&harness, "Copy").expect("the item is painted");
    let wanted = tokens::current(&harness.ctx).highlighted_text;

    assert_eq!(text.fallback_color, wanted);
}
