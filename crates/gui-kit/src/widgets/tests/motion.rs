use std::time::Duration;

use egui::accesskit::Role;
use egui::style::ScrollAnimation;
use egui::{Color32, Key, Rect, ScrollArea, Theme, Ui, ViewportId, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::buttons::ButtonKind;
use super::super::check::partial;
use super::super::chip::chip_look;
use super::super::press::{ItemLook, Pointer};
use super::super::{
    ListItem, RowHeights, RowList, after_skeleton, dimmed, hold_content, pill_button,
    skeleton_rows, switch,
};
use crate::palette::Palette;
use crate::testing::harness;
use crate::theme;
use crate::tokens::{Tokens, mix};

const FRAME: f32 = 1.0 / 60.0;

struct Item(String);

impl ListItem for Item {
    fn key(&self) -> &str {
        &self.0
    }

    fn name(&self) -> &str {
        &self.0
    }
}

pub(super) fn repaint_delay<State>(harness: &Harness<'_, State>) -> Duration {
    harness
        .output()
        .viewport_output
        .get(&ViewportId::ROOT)
        .map_or(Duration::MAX, |viewport| viewport.repaint_delay)
}

fn tokens() -> Tokens {
    Tokens::derive(&Palette::fallback(Theme::Dark), Theme::Dark)
}

#[test]
fn a_chip_looks_as_kirigami_draws_it() {
    let tokens = tokens();
    let accent = tokens.accent;
    let look = |checked, pointer| chip_look(&tokens, accent, checked, pointer);

    assert_eq!(
        look(false, Pointer::Rest),
        ItemLook {
            fill: Color32::TRANSPARENT,
            frame: mix(tokens.window, tokens.text, 0.2),
        }
    );
    assert_eq!(look(false, Pointer::Hovered).frame, accent);
    assert_eq!(
        look(true, Pointer::Rest),
        ItemLook {
            fill: accent.gamma_multiply(0.2),
            frame: accent.gamma_multiply(0.9),
        }
    );
    assert_eq!(
        look(true, Pointer::Hovered),
        look(true, Pointer::Rest),
        "a checked chip keeps its look under the pointer"
    );
    for checked in [false, true] {
        assert_eq!(
            look(checked, Pointer::Pressed),
            ItemLook {
                fill: accent.gamma_multiply(0.3),
                frame: accent.gamma_multiply(0.7),
            }
        );
    }
}

fn pill(less_motion: bool) -> Harness<'static> {
    let mut harness = harness(300.0, 100.0)
        .with_step_dt(FRAME)
        .with_max_steps(60)
        .build_ui(|ui| {
            pill_button(ui, "Apply", ButtonKind::Soft);
        });

    theme::set_reduced_motion(&harness.ctx, less_motion);
    harness.run();
    harness.get_by_role_and_label(Role::Button, "Apply").hover();

    // The hover itself asks for one more frame.
    for _ in 0..3_u8 {
        harness.step();
    }

    harness
}

#[test]
fn a_pill_fades_its_hover_in() {
    let mut harness = pill(false);

    assert_eq!(repaint_delay(&harness), Duration::ZERO, "the fade runs");
    let frames = harness.run();
    assert!(frames <= 8, "about 80 ms: {frames}");
}

#[test]
fn with_less_motion_a_pill_hovers_at_once() {
    let harness = pill(true);

    assert_eq!(repaint_delay(&harness), Duration::MAX);
}

#[test]
fn the_check_mark_draws_from_its_start_to_its_end() {
    let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(16.0, 16.0));

    assert_eq!(partial(rect, 0.0).len(), 1);
    assert_eq!(partial(rect, 1.0).len(), 3);

    let half = partial(rect, 0.5);
    let whole = partial(rect, 1.0);
    let (Some(end), Some(last)) = (half.last(), whole.last()) else {
        panic!("points");
    };

    assert!(end.x < last.x, "{end:?} short of {last:?}");
}

#[test]
fn a_row_fades_into_its_busy_dim_and_back() {
    let mut harness = harness(300.0, 100.0)
        .with_step_dt(FRAME)
        .with_max_steps(60)
        .build_ui_state(
            |ui, (busy, seen): &mut (bool, f32)| {
                *seen = dimmed(ui, *busy, |ui| ui.opacity());
            },
            (false, 0.0),
        );

    harness.run();
    harness.state_mut().0 = true;
    harness.step();
    let fading = harness.state().1;
    assert!(fading < 1.0 && fading > 0.55, "{fading}");

    harness.run();
    assert!((harness.state().1 - 0.55).abs() < 0.01);

    theme::set_reduced_motion(&harness.ctx, true);
    harness.state_mut().0 = false;
    harness.step();
    assert!((harness.state().1 - 1.0).abs() < f32::EPSILON);
    assert_eq!(repaint_delay(&harness), Duration::MAX);
}

/// One list whose switches flip on a click or Space, the rows' state.
fn switches(ui: &mut Ui, (items, on): &mut (Vec<Item>, Vec<bool>)) {
    ScrollArea::vertical().show(ui, |ui| {
        RowList::new("rows", "Rows", RowHeights::Fixed(44.0)).show(ui, items, |ui, item, row| {
            let Some(state) = on.get_mut(row.index) else {
                return;
            };

            if switch(ui, *state, &format!("Switch {}", item.0)).clicked() || row.toggle {
                *state = !*state;
            }
        });
    });
}

fn rows(count: usize, height: f32) -> Harness<'static, (Vec<Item>, Vec<bool>)> {
    let items = (0..count)
        .map(|index| Item(format!("row {index:02}")))
        .collect();

    let mut harness = harness(300.0, height)
        .with_step_dt(FRAME)
        .with_max_steps(60)
        .build_ui_state(switches, (items, vec![false; count]));

    // The harness turns egui's own motion off, these tests watch it.
    harness.ctx.all_styles_mut(|style| {
        style.animation_time = theme::ANIMATION_TIME;
        style.scroll_animation = ScrollAnimation::default();
    });
    harness.run();
    harness
}

#[test]
fn space_flips_a_switch_without_a_slide_and_a_click_slides() {
    let mut harness = rows(2, 200.0);
    harness
        .get_by_role_and_label(Role::ListItem, "row 00")
        .focus();
    harness.run();

    harness.key_press(Key::Space);
    harness.step();
    harness.step();
    assert_eq!(harness.state().1.first(), Some(&true));
    harness.step();
    assert_eq!(repaint_delay(&harness), Duration::MAX, "no slide");

    harness
        .get_by_role_and_label(Role::CheckBox, "Switch row 01")
        .click();
    harness.step();
    harness.step();
    assert_eq!(harness.state().1.get(1), Some(&true));
    harness.step();
    assert_eq!(repaint_delay(&harness), Duration::ZERO, "the slide runs");
}

#[test]
fn an_arrow_scrolls_its_row_into_view_at_once() {
    let mut harness = rows(30, 200.0);
    harness
        .get_by_role_and_label(Role::ListItem, "row 00")
        .click();
    harness.run();

    for _ in 0..6_u8 {
        harness.key_press(Key::ArrowDown);
        harness.step();
        harness.step();
    }

    let row = harness.get_by_role_and_label(Role::ListItem, "row 06");
    assert!(row.is_focused());
    assert!(row.rect().bottom() <= 200.0, "{:?}", row.rect());
    harness.run();
    assert_eq!(repaint_delay(&harness), Duration::MAX, "no scroll left");
}

/// A page that shows a skeleton until `loaded`, then its content's
/// opacity.
fn page(ui: &mut Ui, (loaded, seen): &mut (bool, f32)) {
    let id = ui.make_persistent_id("content");
    if *loaded {
        *seen = after_skeleton(ui, id, |ui| ui.opacity());
    } else {
        hold_content(ui.ctx(), id);
        skeleton_rows(ui, 2, 40.0, "Reading…");
    }
}

fn loading(loaded: bool, less_motion: bool) -> Harness<'static, (bool, f32)> {
    let mut harness = harness(300.0, 200.0)
        .with_step_dt(FRAME)
        .with_max_steps(60)
        .build_ui_state(page, (loaded, 0.0));

    theme::set_reduced_motion(&harness.ctx, less_motion);
    harness.run();
    harness
}

#[test]
fn content_fades_in_where_its_skeleton_was() {
    let mut harness = loading(false, false);

    harness.state_mut().0 = true;
    harness.step();
    let fading = harness.state().1;
    assert!(fading > 0.0 && fading < 1.0, "{fading}");

    harness.run();
    assert!((harness.state().1 - 1.0).abs() < f32::EPSILON);
}

#[test]
fn content_there_from_the_start_and_with_less_motion_shows_at_once() {
    let harness = loading(true, false);
    assert!((harness.state().1 - 1.0).abs() < f32::EPSILON);

    let mut still = loading(false, true);
    still.state_mut().0 = true;
    still.step();
    assert!((still.state().1 - 1.0).abs() < f32::EPSILON);
    assert_eq!(repaint_delay(&still), Duration::MAX);
}

#[test]
fn a_pill_is_as_wide_as_its_layout_reserves() {
    for label in ["OK", "Remove", "Update in Console"] {
        let (drawn, reserved) = (std::cell::Cell::new(0.0), std::cell::Cell::new(0.0));
        let mut harness = harness(400.0, 80.0).build_ui(|ui| {
            reserved.set(super::super::buttons::pill_width(ui, label));
            drawn.set(pill_button(ui, label, ButtonKind::Soft).rect.width());
        });

        harness.run();
        drop(harness);
        let (drawn, reserved) = (drawn.get(), reserved.get());
        assert!(
            (drawn - reserved).abs() < 0.5,
            "{label}: drawn {drawn}, reserved {reserved}"
        );
    }
}
