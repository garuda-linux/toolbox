use egui::accesskit::Role;
use egui::emath::GuiRounding as _;
use egui::{Rect, pos2};
use egui_kittest::kittest::Queryable as _;

use super::super::card::{HOVER_INSET, fill_rect};
use super::super::{
    CheckRowTexts, ListItem, RowHeights, RowList, card, card_row, check_row_with_line,
};
use crate::testing::{filled_rect, filled_rects, harness, lines_in};
use crate::tokens;

struct Item(&'static str);

impl ListItem for Item {
    fn key(&self) -> &str {
        self.0
    }

    fn name(&self) -> &str {
        self.0
    }
}

#[test]
fn a_short_row_in_a_fixed_list_is_centred_in_it() {
    let mut harness = harness(400.0, 300.0).build_ui_state(
        |ui, text: &mut Rect| {
            let items = [Item("a")];
            RowList::new("rows", "Rows", RowHeights::Fixed(120.0)).show(ui, &items, |ui, _, _| {
                *text = ui.label("Firefox").rect;
            });
        },
        Rect::NOTHING,
    );

    harness.run();
    let row = harness.get_by_role_and_label(Role::ListItem, "a").rect();
    let text = *harness.state();

    assert!(
        (row.center().y - text.center().y).abs() < 1.0,
        "row {row:?}, text {text:?}"
    );
    assert!((row.height() - 120.0).abs() < 0.5, "row {row:?}");
}

#[test]
fn the_fill_keeps_the_inset_from_both_dividers_on_any_offset() {
    for top in [100.0, 283.5, 497.75] {
        let row = Rect::from_min_max(pos2(0.0, top), pos2(400.0, top + 46.0));
        let fill = fill_rect(row, 1.0);
        let above = row.top();
        let below = row.bottom().round_to_pixel_center(1.0) - 0.5;

        assert!(
            (fill.top() - above - HOVER_INSET).abs() < 0.01,
            "{top}: {fill:?}"
        );
        assert!(
            (below - fill.bottom() - HOVER_INSET).abs() < 0.01,
            "{top}: {fill:?}"
        );
    }
}

#[test]
fn a_hovered_row_has_the_same_gap_to_the_divider_above_and_below() {
    let mut harness = harness(400.0, 300.0).build_ui(|ui| {
        let items = [Item("a"), Item("b"), Item("c")];
        RowList::new("rows", "Rows", RowHeights::Fixed(46.0)).show(ui, &items, |ui, item, _| {
            ui.label(item.0);
        });
    });

    harness.run();
    let middle = harness.get_by_role_and_label(Role::ListItem, "b").rect();
    harness.hover_at(middle.center());
    harness.run();

    let theme = tokens::current(&harness.ctx);
    let fill = filled_rect(&harness, theme.field).expect("a hover fill");
    let mut lines: Vec<f32> = lines_in(&harness, theme.outline)
        .iter()
        .map(|[start, _]| start.y)
        .collect();

    lines.sort_by(f32::total_cmp);

    let [above, below] = lines[..] else {
        panic!("two dividers expected, got {lines:?}");
    };

    let gap_above = fill.top() - (above + 0.5);
    let gap_below = (below - 0.5) - fill.bottom();
    assert!(
        (gap_above - gap_below).abs() < 0.01,
        "{gap_above} above, {gap_below} below"
    );
}

#[test]
fn a_check_row_on_a_card_row_shows_only_the_card_rows_fill() {
    let mut harness = harness(400.0, 200.0).build_ui(|ui| {
        card(ui, |ui| {
            card_row(ui, true, |ui| {
                let texts = CheckRowTexts {
                    label: "Kitty",
                    line: "Replaces ~/.config/kitty/kitty.conf",
                    full: "~/.config/kitty/kitty.conf",
                };

                _ = check_row_with_line(ui, true, texts, true);
            });
        });
    });

    harness.run();
    let row = harness
        .get_by_role_and_label(Role::CheckBox, "Kitty")
        .rect();

    let point = row.right_center() - egui::vec2(20.0, 0.0);
    harness.hover_at(point);
    harness.run();

    let theme = tokens::current(&harness.ctx);
    let own = filled_rects(&harness, theme.hover)
        .into_iter()
        .filter(|rect| rect.contains(point))
        .count();

    assert_eq!(own, 0, "the check row painted a fill of its own");
    assert!(
        filled_rect(&harness, theme.field).is_some(),
        "no card row fill"
    );
}
