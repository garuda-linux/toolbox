use egui::accesskit::Role;
use egui::{Event, Pos2, Rect, Sense, Vec2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::buttons::SMALL_BUTTON;
use super::super::corner_button;
use crate::testing::{filled_rects, harness, tab_into};
use crate::{icons, theme, tokens};

const NAME: &str = "Copy code";
const AREA: Vec2 = vec2(300.0, 40.0);
const FRAME: f32 = 1.0 / 60.0;

struct Shown {
    area: Rect,
    clicks: usize,
}

fn block(less_motion: bool) -> Harness<'static, Shown> {
    let mut harness = harness(400.0, 200.0)
        .with_step_dt(FRAME)
        .with_max_steps(30)
        .build_ui_state(
            |ui, shown: &mut Shown| {
                let (area, _) = ui.allocate_exact_size(AREA, Sense::hover());
                let middle = area.top() + 12.0;
                if corner_button(ui, area, middle, icons::EDIT_COPY, NAME).clicked() {
                    shown.clicks += 1;
                }

                shown.area = area;
            },
            Shown {
                area: Rect::NOTHING,
                clicks: 0,
            },
        );

    theme::set_reduced_motion(&harness.ctx, less_motion);
    harness.run();
    harness
}

fn backing(harness: &Harness<'_, Shown>) -> Option<Rect> {
    let field = tokens::current(&harness.ctx).field;
    filled_rects(harness, field)
        .into_iter()
        .find(|rect| (rect.width() - SMALL_BUTTON).abs() < 0.01)
}

fn point_at(harness: &Harness<'_, Shown>, at: Pos2) {
    harness.event(Event::PointerMoved(at));
}

#[test]
fn at_rest_it_is_hidden_but_named_for_screen_readers() {
    let harness = block(false);

    assert_eq!(backing(&harness), None);
    assert!(
        harness
            .query_by_role_and_label(Role::Button, NAME)
            .is_some()
    );
}

#[test]
fn the_pointer_anywhere_on_the_area_shows_it_in_the_top_right_corner() {
    let mut harness = block(false);
    let area = harness.state().area;

    point_at(&harness, area.left_center() + vec2(10.0, 0.0));
    harness.run();

    let shown = backing(&harness).expect("the button shows");
    assert!((area.right() - shown.right() - (shown.top() - area.top())).abs() < 0.01);
    assert!((shown.center().y - (area.top() + 12.0)).abs() < 0.01);
}

#[test]
fn it_leaves_with_the_pointer() {
    let mut harness = block(false);
    let area = harness.state().area;

    point_at(&harness, area.center());
    harness.run();
    point_at(&harness, area.max + vec2(40.0, 40.0));
    harness.run();

    assert_eq!(backing(&harness), None);
}

#[test]
fn tab_reaches_it_while_hidden_and_shows_it() {
    let mut harness = block(false);

    assert!(tab_into(&mut harness, Role::Button), "Tab never reaches it");
    harness.run();

    assert!(backing(&harness).is_some(), "the focused button shows");
}

#[test]
fn a_shown_button_takes_a_click() {
    let mut harness = block(false);
    let area = harness.state().area;

    point_at(&harness, area.center());
    harness.run();
    harness.get_by_role_and_label(Role::Button, NAME).click();
    harness.run();

    assert_eq!(harness.state().clicks, 1);
}

#[test]
fn it_fades_in_and_appears_at_once_with_less_motion() {
    for (less_motion, at_once) in [(false, false), (true, true)] {
        let mut harness = block(less_motion);
        let area = harness.state().area;

        point_at(&harness, area.center());
        harness.step();
        harness.step();

        assert_eq!(
            backing(&harness).is_some(),
            at_once,
            "less motion {less_motion}"
        );
    }
}
